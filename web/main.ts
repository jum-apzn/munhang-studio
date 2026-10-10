import "./style.css";
import {
  blankProject,
  LIMITS,
  demoProject,
  parseProject,
  validateExportProject,
  reorder,
  normalizedRect,
  dataUrlBytes,
  escapeHtml as e,
} from "./utils.ts";
import type { Project, Block, Rect } from "./utils.ts";
declare global {
  interface Window {
    __TAURI__?: {
      core: {
        invoke: <T>(
          command: string,
          args?: Record<string, unknown>,
        ) => Promise<T>;
      };
    };
  }
}
const invoke = window.__TAURI__?.core.invoke;
let project: Project = blankProject();
let recovered = false;
let recoveryRaw: string | null = null;
let recoveryError = "";
try {
  const saved = localStorage.getItem("munhang-project-v1");
  if (saved) {
    recoveryRaw = saved;
    project = parseProject(saved);
    recoveryRaw = null;
    recovered = true;
  }
} catch (error) {
  recoveryError =
    "저장된 작업을 복원하지 못했습니다. 원본은 보존했으며 덮어쓰지 않습니다. 복구 원본을 확인해 주세요. " +
    String(error instanceof Error ? error.message : error);
}
let lastValidSnapshot = JSON.stringify(project);
let selected = project.items[0]?.id ?? "",
  pages: string[] = [],
  pageIndex = 0,
  sourceName = "",
  selection: Rect | null = null,
  busy = false;
let tab: "source" | "preview" = "source";
let runtime = { pdf: false, ocr: false, korean: false, formula: false };
let message =
  recoveryError ||
  (recovered
    ? "이 브라우저에 저장된 마지막 작업을 복원했어요."
    : "원본을 불러오거나 예제로 가볍게 시작해 보세요.");
let tone = recoveryError ? "error" : "info";
const app = document.querySelector<HTMLDivElement>("#app")!;
const current = () => project.items.find((i) => i.id === selected);
const icon = (name: string) =>
  ({ file: "▤", plus: "＋", export: "↗", check: "✓", image: "▧" })[name] ??
  name;
function save() {
  if (recoveryError) {
    document.querySelector("#save-state")!.textContent =
      "복구 보호 · 자동 저장 중지";
    return;
  }
  try {
    parseProject(JSON.stringify(project));
    localStorage.setItem("munhang-project-v1", JSON.stringify(project));
    document.querySelector("#save-state")!.textContent = "이 기기에 저장됨";
  } catch (error) {
    document.querySelector("#save-state")!.textContent =
      "자동 저장 실패 · 기존 저장 보존 · JSON 저장 필요";
    notify(
      "저장 공간 부족 또는 프로젝트 형식 오류로 자동 저장하지 못했습니다. 기존 저장은 보존됩니다.",
      true,
    );
  }
}
function notify(text: string, error = false) {
  message = text;
  tone = error ? "error" : "info";
  const el = document.querySelector("#notice");
  if (el) {
    el.textContent = text;
    el.className = `notice ${tone}`;
  }
}
function update() {
  // A transaction boundary for every field/reorder edit: reject and restore the
  // last valid in-memory state before any persistence or rendered preview.
  try {
    parseProject(JSON.stringify(project));
    lastValidSnapshot = JSON.stringify(project);
  } catch (error) {
    project = JSON.parse(lastValidSnapshot) as Project;
    if (!current()) selected = project.items[0]?.id ?? "";
    shell();
    notify(
      "변경을 적용하지 않았습니다: " +
        String(error instanceof Error ? error.message : error),
      true,
    );
    return;
  }
  save();
  renderList();
  renderPreview();
  document.querySelector("#count")!.textContent = String(project.items.length);
  document.querySelector("#total")!.textContent = String(
    project.items.reduce((sum, i) => sum + i.points, 0),
  );
}
function shell() {
  app.innerHTML = `
<header class="topbar"><div class="brand"><span class="brand-icon">문</span><div>문항공방<small>WORKSHEET STUDIO</small></div></div><div class="top-meta"><span class="local-dot"></span>내 컴퓨터에서 안전하게 <span class="divider">|</span> <span id="save-state">${recoveryError ? "복구 보호 · 자동 저장 중지" : recovered ? "복원 완료" : "새 프로젝트"}</span></div><div class="actions"><button id="load-project">프로젝트 열기</button><button id="save-project">JSON 저장</button><button class="primary" id="export">HWPX 내보내기 ${icon("export")}</button></div></header>
${recoveryError ? `<section class="recovery-panel" role="alert"><strong>저장된 작업 복구 필요 · 원본 보존 중</strong><p>${e(recoveryError)} 현재 편집 내용은 JSON으로 따로 저장해 주세요.</p><button id="show-recovery">복구 원본 보기</button><button id="download-recovery">복구 원본 저장</button><textarea id="recovery-raw" hidden readonly aria-label="보존된 원본 JSON"></textarea></section>` : ""}
<div class="workspace-head"><div><p class="eyebrow">수학 문제지 만들기</p><h1>좋은 문항을, 나만의 문제지로.</h1></div><div class="steps"><span><b>1</b> 원본 선택</span><i>›</i><span><b>2</b> 문항 편집</span><i>›</i><span><b>3</b> 한글로 완성</span></div></div>
<main class="workspace"><aside class="library panel"><div class="panel-heading"><div><span class="eyebrow">MY QUESTIONS</span><h2>문항 보관함 <span id="count">${project.items.length}</span></h2></div><button class="icon-btn" id="add-question" title="빈 문항 추가" aria-label="빈 문항 추가">＋</button></div><div id="question-list" class="question-list"></div><div class="library-bottom"><button id="demo" class="wide">예제 문항으로 시작</button><p>직접 만든 예제 2문항을 불러옵니다.</p><div class="stat"><span>총 배점</span><strong><span id="total">${project.items.reduce((s, i) => s + i.points, 0)}</span> 점</strong></div></div></aside>
<section class="canvas-panel panel"><div class="canvas-toolbar"><div class="tabs"><button id="tab-source" class="${tab === "source" ? "active" : ""}">원본 & 영역 선택</button><button id="tab-preview" class="${tab === "preview" ? "active" : ""}">문제지 미리보기</button></div><button id="import-source">＋ 원본 불러오기</button></div><div class="source-meta"><span id="source-name">${e(sourceName || "PDF · PNG · JPEG")}</span><div id="page-controls"></div></div><div id="canvas-content" class="canvas-content"></div><div id="crop-actions" class="crop-actions"></div><div class="canvas-foot"><span>⌁ 파일은 외부 서버로 전송되지 않습니다</span><span id="runtime-state">${invoke ? "로컬 도구 확인 중…" : "브라우저 미리보기 모드"}</span></div></section>
<aside class="editor panel"><div class="panel-heading"><div><span class="eyebrow">DETAIL EDITOR</span><h2>문항 편집</h2></div><span class="edit-badge">편집 가능</span></div><div class="editor-scroll"><section class="document-settings"><h3>문제지 설정</h3><label>제목<input id="title" maxlength="500" value="${e(project.title)}"></label><div class="field-row"><label>단 구성<select id="columns"><option value="1" ${project.columns === 1 ? "selected" : ""}>1단</option><option value="2" ${project.columns === 2 ? "selected" : ""}>2단</option></select></label><label>시작 번호<input id="start-number" type="number" min="1" max="9999" value="${project.startNumber}"></label></div></section><div id="item-editor"></div></div></aside></main><div id="notice" class="notice ${tone}" role="status" aria-live="polite">${e(message)}</div><footer>문항공방 <span>원본의 저작권과 이용 범위를 확인해 주세요.</span><span>LOCAL-FIRST · v0.1</span></footer>
<input id="source-input" type="file" accept="image/png,image/jpeg,application/pdf,.pdf" multiple hidden><input id="project-input" type="file" accept=".json,application/json" hidden>`;
  renderList();
  renderCanvas();
  renderEditor();
  bind();
  showRuntime();
}
function renderList() {
  document.querySelector("#question-list")!.innerHTML = project.items.length
    ? project.items
        .map((item, i) => {
          const text = item.blocks.find((b) => b.kind === "text");
          return `<button class="question-card ${selected === item.id ? "selected" : ""}" data-select="${e(item.id)}"><div class="question-top"><span class="q-number">${String(project.startNumber + i).padStart(2, "0")}</span><span>${item.points}점</span></div><p>${e(text?.kind === "text" ? text.text || "새 문항" : "내용을 추가해 주세요")}</p><div class="question-tags"><span>${item.blocks.length}개 블록</span>${item.blocks.some((b) => b.kind === "equation") ? "<span>수식</span>" : ""}${item.blocks.some((b) => b.kind === "image") ? "<span>그림</span>" : ""}</div></button>`;
        })
        .join("")
    : `<div class="empty-list"><span>▤</span><h3>첫 문항을 담아 보세요</h3><p>원본에서 영역을 선택하거나<br>위의 +로 직접 작성하세요.</p></div>`;
  document.querySelectorAll<HTMLButtonElement>("[data-select]").forEach(
    (b) =>
      (b.onclick = () => {
        selected = b.dataset.select!;
        renderList();
        renderEditor();
      }),
  );
}
function equationReference(
  block: Extract<Block, { kind: "equation" }>,
): string {
  if (
    !block.sourceImage &&
    block.sourceLatex === undefined &&
    !block.conversionWarnings?.length
  )
    return "";
  return `<div class="formula-evidence"><strong>편집용 원본 · 문제지에 출력되지 않음</strong>${block.sourceImage ? `<img class="equation-reference-image" src="${e(block.sourceImage)}" alt="수식 원본 참고 이미지">` : ""}${block.sourceLatex !== undefined ? `<span>인식 원문 · LaTeX</span><pre>${e(block.sourceLatex)}</pre>` : ""}${block.conversionWarnings?.length ? `<p>${e(block.conversionWarnings.join(" / "))}</p>` : ""}<small>원본과 인식 기록은 프로젝트 JSON에 함께 저장됩니다.</small></div>`;
}
function renderEditor() {
  const item = current(),
    i = project.items.findIndex((x) => x.id === selected);
  const el = document.querySelector("#item-editor")!;
  if (!item) {
    el.innerHTML =
      '<div class="editor-empty"><span>✎</span><h3>문항을 선택해 주세요</h3><p>본문, 수식, 그림을 블록 단위로<br>자유롭게 다듬을 수 있어요.</p></div>';
    return;
  }
  el.innerHTML = `<section class="item-settings"><div class="section-title"><h3>문항 ${project.startNumber + i}</h3><div><button id="move-up" class="small-btn" ${i === 0 ? "disabled" : ""} aria-label="문항 위로">↑</button><button id="move-down" class="small-btn" ${i === project.items.length - 1 ? "disabled" : ""} aria-label="문항 아래로">↓</button><button id="delete-question" class="small-btn danger">삭제</button></div></div><div class="field-row"><label>배점<input type="number" id="points" min="0" max="1000" step="0.5" value="${item.points}"></label><label>풀이 여백 (mm)<input type="number" id="answer-space" min="0" max="200" value="${item.answerSpaceMm}"></label></div></section><div class="block-list">${item.blocks
    .map(
      (b, j) =>
        `<div class="block"><div class="block-heading"><span>${b.kind === "text" ? "본문" : b.kind === "equation" ? "한컴 수식" : "그림"} <small>${j + 1}</small></span><div><button data-block-up="${j}" class="small-btn" ${j === 0 ? "disabled" : ""} aria-label="블록 위로">↑</button><button data-remove-block="${j}" class="small-btn" aria-label="블록 삭제">×</button></div></div>${
          b.kind === "text"
            ? `<textarea data-text="${j}" maxlength="${LIMITS.text}" rows="5" placeholder="문제 본문을 입력하세요">${e(b.text)}</textarea>`
            : b.kind === "equation"
              ? `${equationReference(b)}<p class="equation-required" data-equation-required="${j}" ${b.script.trim() ? "hidden" : ""}>수식을 입력해야 HWPX로 내보낼 수 있어요. JSON에는 초안으로 저장할 수 있습니다.</p><textarea class="equation-input" data-equation="${j}" maxlength="${LIMITS.script}" rows="3" placeholder="예: {a} over {b}">${e(b.script)}</textarea><p class="hint">한컴 수식 스크립트 · LaTeX 자동 변환 아님</p><div class="formula-presets">${[
                  ["분수", "{a} over {b}"],
                  ["제곱", "x^2"],
                  ["루트", "sqrt {x}"],
                ]
                  .map(
                    ([label, value]) =>
                      `<button data-preset="${j}" data-script="${e(value)}">${label}</button>`,
                  )
                  .join("")}</div>`
              : `<img class="block-image" src="${e(b.dataUrl)}" alt="${e(b.alt)}"><label>그림 설명<input data-alt="${j}" value="${e(b.alt)}"></label><label>너비 (mm)<input type="number" min="1" max="180" data-width="${j}" value="${b.widthMm}"></label>`
        }</div>`,
    )
    .join(
      "",
    )}</div><div class="add-blocks"><button id="add-text">＋ 본문</button><button id="add-equation">＋ 수식</button></div><p class="editor-note">그림은 원본에서 영역을 선택해 추가하세요.<br>수식은 내보낸 한글 문서에서 최종 확인해 주세요.</p>`;
  (document.querySelector("#move-up") as HTMLButtonElement).onclick = () => {
    project.items = reorder(project.items, i, i - 1);
    update();
    renderEditor();
  };
  (document.querySelector("#move-down") as HTMLButtonElement).onclick = () => {
    project.items = reorder(project.items, i, i + 1);
    update();
    renderEditor();
  };
  (document.querySelector("#delete-question") as HTMLButtonElement).onclick =
    () => {
      if (!confirm("이 문항을 삭제할까요? 이 작업은 되돌릴 수 없습니다."))
        return;
      project.items.splice(i, 1);
      selected = project.items[Math.min(i, project.items.length - 1)]?.id ?? "";
      update();
      renderEditor();
    };
  numberInput("#points", (v) => (item.points = v), 0, 1000);
  numberInput("#answer-space", (v) => (item.answerSpaceMm = v), 0, 200);
  el.querySelectorAll<HTMLTextAreaElement>("[data-text]").forEach(
    (x) =>
      (x.oninput = () => {
        const block = item.blocks[Number(x.dataset.text)] as Extract<
          Block,
          { kind: "text" }
        >;
        if (x.value.length > LIMITS.text) {
          x.value = block.text;
          notify(
            `본문은 ${LIMITS.text.toLocaleString()}자까지 입력할 수 있습니다. 기존 내용을 유지했습니다.`,
            true,
          );
          return;
        }
        (
          item.blocks[Number(x.dataset.text)] as Extract<
            Block,
            { kind: "text" }
          >
        ).text = x.value;
        update();
      }),
  );
  el.querySelectorAll<HTMLTextAreaElement>("[data-equation]").forEach(
    (x) =>
      (x.oninput = () => {
        const block = item.blocks[Number(x.dataset.equation)] as Extract<
          Block,
          { kind: "equation" }
        >;
        if (x.value.length > LIMITS.script) {
          x.value = block.script;
          notify(
            `수식은 ${LIMITS.script.toLocaleString()}자까지 입력할 수 있습니다. 기존 내용을 유지했습니다.`,
            true,
          );
          return;
        }
        (
          item.blocks[Number(x.dataset.equation)] as Extract<
            Block,
            { kind: "equation" }
          >
        ).script = x.value;
        const warning = document.querySelector<HTMLElement>(
          `[data-equation-required="${x.dataset.equation}"]`,
        );
        if (warning) warning.hidden = !!x.value.trim();
        update();
      }),
  );
  el.querySelectorAll<HTMLButtonElement>("[data-remove-block]").forEach(
    (x) =>
      (x.onclick = () => {
        item.blocks.splice(Number(x.dataset.removeBlock), 1);
        update();
        renderEditor();
      }),
  );
  el.querySelectorAll<HTMLButtonElement>("[data-block-up]").forEach(
    (x) =>
      (x.onclick = () => {
        const j = Number(x.dataset.blockUp);
        item.blocks = reorder(item.blocks, j, j - 1);
        update();
        renderEditor();
      }),
  );
  el.querySelectorAll<HTMLButtonElement>("[data-preset]").forEach(
    (x) =>
      (x.onclick = () => {
        const block = item.blocks[Number(x.dataset.preset)] as Extract<
          Block,
          { kind: "equation" }
        >;
        if (
          block.script.length + (x.dataset.script?.length ?? 0) >
          LIMITS.script
        ) {
          notify("수식 길이 제한을 초과해 추가하지 않았습니다.", true);
          return;
        }
        (
          item.blocks[Number(x.dataset.preset)] as Extract<
            Block,
            { kind: "equation" }
          >
        ).script += x.dataset.script;
        update();
        renderEditor();
      }),
  );
  el.querySelectorAll<HTMLInputElement>("[data-width]").forEach(
    (x) =>
      (x.onchange = () => {
        const v = Number(x.value);
        if (v >= 1 && v <= 180) {
          (
            item.blocks[Number(x.dataset.width)] as Extract<
              Block,
              { kind: "image" }
            >
          ).widthMm = v;
          update();
        }
      }),
  );
  el.querySelectorAll<HTMLInputElement>("[data-alt]").forEach(
    (x) =>
      (x.oninput = () => {
        (
          item.blocks[Number(x.dataset.alt)] as Extract<
            Block,
            { kind: "image" }
          >
        ).alt = x.value;
        update();
      }),
  );
  (document.querySelector("#add-text") as HTMLButtonElement).onclick = () =>
    addBlock({ kind: "text", text: "" });
  (document.querySelector("#add-equation") as HTMLButtonElement).onclick = () =>
    addBlock({ kind: "equation", script: "" });
}
function numberInput(
  selector: string,
  set: (n: number) => void,
  min: number,
  max: number,
  integer = false,
) {
  const el = document.querySelector<HTMLInputElement>(selector)!;
  el.onchange = () => {
    const value = Number(el.value);
    if (
      !Number.isFinite(value) ||
      value < min ||
      value > max ||
      (integer && !Number.isInteger(value))
    ) {
      notify("입력 범위를 확인해 주세요.", true);
      return;
    }
    set(value);
    update();
  };
}
function validCandidate(candidate: Project): boolean {
  try {
    parseProject(JSON.stringify(candidate));
    return true;
  } catch (error) {
    notify(
      "추가하지 않았습니다: " +
        String(error instanceof Error ? error.message : error),
      true,
    );
    return false;
  }
}
function addQuestion(blocks: Block[] = [{ kind: "text", text: "" }]): boolean {
  if (project.items.length >= LIMITS.items) {
    notify(
      `문항은 ${LIMITS.items}개까지 추가할 수 있습니다. 기존 문항을 유지했습니다.`,
      true,
    );
    return false;
  }
  const item = {
    id: crypto.randomUUID(),
    points: 4,
    answerSpaceMm: 25,
    blocks,
  };
  if (!validCandidate({ ...project, items: [...project.items, item] }))
    return false;
  project.items.push(item);
  selected = item.id;
  update();
  renderEditor();
  return true;
}
function addBlock(block: Block): boolean {
  const item = current();
  if (!item) return addQuestion([block]);
  if (item.blocks.length >= LIMITS.blocks) {
    notify(
      `문항당 블록은 ${LIMITS.blocks}개까지 추가할 수 있습니다. 기존 내용을 유지했습니다.`,
      true,
    );
    return false;
  }
  if (
    !validCandidate({
      ...project,
      items: project.items.map((i) =>
        i === item ? { ...item, blocks: [...item.blocks, block] } : i,
      ),
    })
  )
    return false;
  item.blocks.push(block);
  update();
  renderEditor();
  return true;
}
function renderCanvas() {
  document
    .querySelector("#tab-source")
    ?.classList.toggle("active", tab === "source");
  document
    .querySelector("#tab-preview")
    ?.classList.toggle("active", tab === "preview");
  document.querySelector("#source-name")!.textContent =
    tab === "preview"
      ? "A4 · 편집용 미리보기"
      : sourceName || "PDF · PNG · JPEG";
  document.querySelector("#page-controls")!.innerHTML =
    tab === "source" && pages.length
      ? `<button id="prev-page" class="small-btn" ${pageIndex === 0 ? "disabled" : ""} aria-label="이전 페이지">‹</button><span>${pageIndex + 1} / ${pages.length}</span><button id="next-page" class="small-btn" ${pageIndex === pages.length - 1 ? "disabled" : ""} aria-label="다음 페이지">›</button>`
      : "";
  if (tab === "preview") {
    renderPreview();
    renderCropActions();
    return;
  }
  const content = document.querySelector("#canvas-content")!;
  if (!pages.length) {
    content.innerHTML = `<div class="source-empty"><div class="paper-symbol"><span>ƒ</span><i></i><i></i><i></i><b>＋</b></div><h2>문항의 시작은 원본에서</h2><p>PDF나 이미지를 불러온 뒤,<br>필요한 부분만 드래그해 선택하세요.</p><button id="empty-import" class="primary">원본 불러오기</button><small>PNG, JPEG · PDF는 데스크톱 앱에서 지원</small><div class="empty-tip"><span>TIP</span> 파일이 없어도 빈 문항을 추가해 직접 작성할 수 있어요.</div></div>`;
    (document.querySelector("#empty-import") as HTMLButtonElement).onclick =
      () => document.querySelector<HTMLInputElement>("#source-input")!.click();
  } else {
    content.innerHTML =
      '<div class="source-sheet"><canvas id="source-canvas" aria-label="원본 이미지. 마우스로 영역을 드래그해 선택하세요."></canvas><div id="selection-box" hidden></div></div>';
    const canvas = document.querySelector<HTMLCanvasElement>("#source-canvas")!;
    const image = new Image();
    image.onload = () => {
      canvas.width = image.naturalWidth;
      canvas.height = image.naturalHeight;
      canvas.getContext("2d")!.drawImage(image, 0, 0);
      bindSelection(canvas);
    };
    image.src = pages[pageIndex];
    for (const [id, delta] of [
      ["prev-page", -1],
      ["next-page", 1],
    ] as const) {
      (document.querySelector("#" + id) as HTMLButtonElement).onclick = () => {
        pageIndex += delta;
        selection = null;
        renderCanvas();
      };
    }
  }
  renderCropActions();
}
function renderPreview() {
  if (tab !== "preview") return;
  document.querySelector("#canvas-content")!.innerHTML =
    `<div class="preview-note">배치 참고용입니다. 수식은 스크립트로 표시하며 실제 HWPX의 줄바꿈과 다를 수 있어요.</div><article class="exam-sheet"><div class="exam-topline"><span>MATHEMATICS</span><span>이름 __________________</span></div><h2>${e(project.title)}</h2><div class="exam-rule"></div><div class="exam-questions cols-${project.columns}">${project.items.length ? project.items.map((q, i) => `<section class="exam-question"><div class="exam-q-head"><strong>${project.startNumber + i}.</strong><span>[${q.points}점]</span></div>${q.blocks.map((b) => (b.kind === "text" ? `<p>${e(b.text).replace(/\n/g, "<br>")}</p>` : b.kind === "equation" ? `<pre class="exam-equation">${e(b.script)}</pre>` : `<img src="${e(b.dataUrl)}" alt="${e(b.alt)}" style="width:${b.widthMm}mm;max-width:100%">`)).join("")}<div class="answer-space" style="height:${q.answerSpaceMm}mm"></div></section>`).join("") : '<p class="preview-placeholder">문항을 추가하면 여기에 표시됩니다.</p>'}</div></article>`;
}
function bindSelection(canvas: HTMLCanvasElement) {
  let start: { x: number; y: number } | null = null;
  const box = document.querySelector<HTMLDivElement>("#selection-box")!;
  const point = (ev: PointerEvent) => {
    const r = canvas.getBoundingClientRect();
    return {
      x: ((ev.clientX - r.left) * canvas.width) / r.width,
      y: ((ev.clientY - r.top) * canvas.height) / r.height,
    };
  };
  const draw = () => {
    if (!selection) return;
    box.hidden = false;
    box.style.left = (selection.x / canvas.width) * 100 + "%";
    box.style.top = (selection.y / canvas.height) * 100 + "%";
    box.style.width = (selection.width / canvas.width) * 100 + "%";
    box.style.height = (selection.height / canvas.height) * 100 + "%";
  };
  canvas.onpointerdown = (ev) => {
    if (ev.button !== 0) return;
    start = point(ev);
    canvas.setPointerCapture(ev.pointerId);
    selection = null;
  };
  canvas.onpointermove = (ev) => {
    if (!start) return;
    const p = point(ev);
    selection = normalizedRect(
      start.x,
      start.y,
      p.x,
      p.y,
      canvas.width,
      canvas.height,
    );
    draw();
  };
  canvas.onpointerup = (ev) => {
    if (!start) return;
    const p = point(ev);
    selection = normalizedRect(
      start.x,
      start.y,
      p.x,
      p.y,
      canvas.width,
      canvas.height,
    );
    start = null;
    if (selection.width < 8 || selection.height < 8) {
      selection = null;
      box.hidden = true;
    }
    draw();
    renderCropActions();
  };
  canvas.onpointercancel = () => {
    start = null;
    selection = null;
    box.hidden = true;
    renderCropActions();
  };
}
function renderCropActions() {
  const bar = document.querySelector("#crop-actions")!;
  if (tab !== "source") {
    bar.innerHTML = "";
    return;
  }
  bar.innerHTML = `<span>${selection ? `${Math.round(selection.width)} × ${Math.round(selection.height)} px 선택됨` : "원본에서 필요한 영역을 드래그하세요"}</span><div><button id="crop-question" ${!selection || busy ? "disabled" : ""}>새 문항</button><button id="crop-text" ${!selection || busy ? "disabled" : ""}>본문 OCR</button><button id="crop-equation" ${!selection || busy ? "disabled" : ""}>수식 직접 입력</button><button id="crop-formula" ${!selection || busy ? "disabled" : ""}>수식 OCR</button><button id="crop-image" ${!selection || busy ? "disabled" : ""}>그림</button></div>`;
  for (const kind of ["question", "text", "equation", "formula", "image"])
    (document.querySelector("#crop-" + kind) as HTMLButtonElement).onclick =
      () => void useCrop(kind);
}
function crop(): string {
  if (!selection) throw new Error("먼저 원본 영역을 선택해 주세요.");
  const src = document.querySelector<HTMLCanvasElement>("#source-canvas")!;
  const c = document.createElement("canvas");
  c.width = Math.round(selection.width);
  c.height = Math.round(selection.height);
  c.getContext("2d")!.drawImage(
    src,
    selection.x,
    selection.y,
    selection.width,
    selection.height,
    0,
    0,
    c.width,
    c.height,
  );
  return c.toDataURL("image/png");
}
async function useCrop(kind: string) {
  if (busy) {
    notify("진행 중인 작업이 끝나면 다시 시도해 주세요.");
    return;
  }
  const originProject = project,
    originItem = selected;
  const verifyTarget = () => {
    if (project !== originProject || selected !== originItem)
      throw new Error(
        "작업 중 편집 대상이 바뀌어 인식 결과를 적용하지 않았습니다. 원하는 문항을 선택해 다시 실행해 주세요.",
      );
  };
  try {
    const dataUrl = crop();
    if (kind === "formula") {
      if (!invoke || !runtime.formula)
        throw new Error(
          "수식 OCR은 로컬 수식 인식 모델을 설치한 데스크톱 앱에서 사용할 수 있어요. 설치 안내를 확인하거나 수식 직접 입력을 사용해 주세요.",
        );
      busy = true;
      renderCropActions();
      notify("선택한 수식을 이 컴퓨터에서 인식하고 있어요…");
      const result = await invoke<{
        latex: string;
        script: string;
        warnings: string[];
      }>("recognize_formula", { data: dataUrlBytes(dataUrl) });
      verifyTarget();
      if (
        !addBlock({
          kind: "equation",
          script: result.script,
          sourceImage: dataUrl,
          sourceLatex: result.latex,
          conversionWarnings: result.warnings,
        })
      )
        return;
      renderEditor();
      notify(
        !result.script.trim()
          ? "수식 원본과 인식 결과를 보관했어요. 변환을 완료하지 못했으니 스크립트를 직접 입력해 주세요. JSON 초안 저장은 가능합니다."
          : result.warnings.length
            ? "수식 인식 완료. 변환 경고가 있으니 원문과 수식 스크립트를 확인해 주세요."
            : "수식 인식 완료. 첨자와 기호를 원본과 비교해 주세요.",
      );
      return;
    }
    if (kind === "image") {
      if (
        !addBlock({
          kind: "image",
          dataUrl,
          widthMm: 70,
          alt: "원본에서 선택한 그림",
        })
      )
        return;
      notify("선택한 영역을 원본 해상도의 그림으로 추가했어요.");
      return;
    }
    if (kind === "equation") {
      if (!addBlock({ kind: "equation", script: "", sourceImage: dataUrl }))
        return;
      notify(
        "수식 원본과 빈 수식 블록을 추가했어요. 원본을 보며 한컴 수식 스크립트를 입력해 주세요.",
      );
      return;
    }
    if (kind === "question") {
      if (
        !addQuestion([
          { kind: "image", dataUrl, widthMm: 75, alt: "선택한 문항 원본" },
          { kind: "text", text: "" },
        ])
      )
        return;
      notify(
        "새 문항에 원본을 담았어요. 본문을 직접 입력하거나 선택 영역에 본문 OCR을 실행하세요.",
      );
      return;
    }
    if (!invoke || !runtime.ocr)
      throw new Error(
        "본문 OCR은 Tesseract가 설치된 데스크톱 앱에서 사용할 수 있어요. 본문을 직접 입력해 주세요.",
      );
    busy = true;
    renderCropActions();
    notify("선택한 영역을 이 컴퓨터에서 인식하고 있어요…");
    const text = await invoke<string>("recognize_image", {
      data: dataUrlBytes(dataUrl),
    });
    verifyTarget();
    if (!text.trim())
      throw new Error(
        "인식된 글자가 없습니다. 더 선명하거나 넓은 영역을 선택해 보세요.",
      );
    if (!addBlock({ kind: "text", text })) return;
    notify("본문 인식 완료. 숫자, 기호와 줄바꿈을 원본과 비교해 주세요.");
  } catch (err) {
    notify(String(err instanceof Error ? err.message : err), true);
  } finally {
    busy = false;
    renderCropActions();
  }
}
function download(data: Blob, name: string) {
  const url = URL.createObjectURL(data),
    a = document.createElement("a");
  a.href = url;
  a.download = name;
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 5000);
}
function name(ext: string) {
  return (project.title || "문제지").replace(/[\\/:*?"<>|]/g, "_") + "." + ext;
}
async function importFiles(files: FileList | null) {
  if (busy) {
    notify("진행 중인 작업이 끝나면 다시 시도해 주세요.");
    return;
  }
  if (!files?.length) return;
  busy = true;
  notify("원본을 불러오고 있어요…");
  try {
    const next: string[] = [];
    for (const file of Array.from(files)) {
      if (file.size > 60 * 1024 * 1024)
        throw new Error("파일 하나의 크기는 60MB 이하여야 합니다.");
      if (file.type === "application/pdf" || /\.pdf$/i.test(file.name)) {
        if (!invoke)
          throw new Error(
            "PDF 불러오기는 데스크톱 앱에서 지원합니다. 브라우저에서는 PNG 또는 JPEG를 사용해 주세요.",
          );
        if (!runtime.pdf)
          throw new Error(
            "PDF 변환 도구(pdftoppm)가 없습니다. 설치 안내를 확인하거나 이미지로 불러와 주세요.",
          );
        next.push(
          ...(await invoke<string[]>("import_pdf", {
            data: Array.from(new Uint8Array(await file.arrayBuffer())),
          })),
        );
      } else if (["image/png", "image/jpeg"].includes(file.type)) {
        const url = await new Promise<string>((resolve, reject) => {
          const r = new FileReader();
          r.onload = () => resolve(String(r.result));
          r.onerror = () => reject(new Error("파일 읽기에 실패했습니다."));
          r.readAsDataURL(file);
        });
        await new Promise<void>((resolve, reject) => {
          const image = new Image();
          image.onload = () =>
            image.naturalWidth * image.naturalHeight > 60_000_000
              ? reject(
                  new Error(
                    "이미지가 너무 큽니다. 6천만 화소 이하로 줄여 주세요.",
                  ),
                )
              : resolve();
          image.onerror = () =>
            reject(new Error("이미지 파일이 손상되었습니다."));
          image.src = url;
        });
        next.push(url);
      } else throw new Error("PNG, JPEG, PDF 파일만 불러올 수 있습니다.");
    }
    if (!next.length) throw new Error("불러올 수 있는 페이지가 없습니다.");
    pages = next;
    pageIndex = 0;
    selection = null;
    sourceName = files.length === 1 ? files[0].name : `원본 ${files.length}개`;
    tab = "source";
    renderCanvas();
    notify(
      `${pages.length}페이지를 불러왔어요. 필요한 영역을 드래그해 보세요. 원본 전체는 프로젝트 JSON에 저장되지 않습니다.`,
    );
  } catch (err) {
    notify(String(err instanceof Error ? err.message : err), true);
  } finally {
    busy = false;
    renderCropActions();
  }
}
function showRuntime() {
  const el = document.querySelector("#runtime-state");
  if (el)
    el.textContent = invoke
      ? `PDF ${runtime.pdf ? "준비됨" : "도구 없음"} · OCR ${runtime.ocr ? (runtime.korean ? "한국어 준비됨" : "한국어 팩 없음") : "도구 없음"} · 수식 ${runtime.formula ? "준비됨" : "모델 없음"}`
      : "브라우저 모드 · HWPX / OCR은 데스크톱 앱";
}
function bind() {
  if (recoveryError) {
    const raw = document.querySelector<HTMLTextAreaElement>("#recovery-raw")!;
    raw.value =
      recoveryRaw ??
      "저장소에 접근하지 못했습니다. 브라우저의 저장소 권한을 확인해 주세요.";
    (document.querySelector("#show-recovery") as HTMLButtonElement).onclick =
      () => {
        raw.hidden = !raw.hidden;
        if (!raw.hidden) {
          raw.focus();
          raw.select();
        }
      };
    (
      document.querySelector("#download-recovery") as HTMLButtonElement
    ).onclick = async () => {
      if (recoveryRaw === null) {
        notify(
          "읽을 수 있는 복구 원본이 없습니다. 저장소 권한을 확인해 주세요.",
          true,
        );
        return;
      }
      const button =
        document.querySelector<HTMLButtonElement>("#download-recovery")!;
      button.disabled = true;
      try {
        if (invoke) {
          const path = await invoke<string | null>(
            "save_recovery_text_to_file",
            { text: recoveryRaw },
          );
          notify(
            path
              ? "복구 원본을 그대로 저장했습니다: " + path
              : "복구 원본 저장을 취소했습니다. 원본은 계속 보존됩니다.",
          );
        } else {
          download(
            new Blob([recoveryRaw], { type: "application/json" }),
            "munhang-recovery-original.json",
          );
          notify(
            "보존된 복구 원본 다운로드를 시작했습니다. 기존 저장은 계속 보존됩니다.",
          );
        }
      } catch (error) {
        notify(
          "복구 원본 저장 실패: " +
            String(error) +
            " 복구 원본 보기에서 내용을 복사할 수 있습니다.",
          true,
        );
      } finally {
        button.disabled = false;
      }
    };
  }

  (document.querySelector("#add-question") as HTMLButtonElement).onclick = () =>
    addQuestion();
  (document.querySelector("#demo") as HTMLButtonElement).onclick = () => {
    if (
      project.items.length &&
      !confirm(
        "현재 문제지를 예제로 바꿀까요? 필요한 작업은 먼저 JSON으로 저장해 주세요.",
      )
    )
      return;
    project = demoProject();
    selected = project.items[0].id;
    tab = "preview";
    shell();
    update();
    notify("직접 만든 예제 문항입니다. 본문과 수식을 자유롭게 바꿔 보세요.");
  };
  (document.querySelector("#tab-source") as HTMLButtonElement).onclick = () => {
    tab = "source";
    selection = null;
    renderCanvas();
  };
  (document.querySelector("#tab-preview") as HTMLButtonElement).onclick =
    () => {
      tab = "preview";
      renderCanvas();
    };
  (document.querySelector("#title") as HTMLInputElement).oninput = (ev) => {
    const input = ev.target as HTMLInputElement;
    if (input.value.length > LIMITS.title) {
      input.value = project.title;
      notify("제목은 500자까지 입력할 수 있습니다.", true);
      return;
    }
    project.title = input.value;
    update();
  };
  (document.querySelector("#columns") as HTMLSelectElement).onchange = (ev) => {
    project.columns = Number((ev.target as HTMLSelectElement).value) as 1 | 2;
    update();
  };
  numberInput(
    "#start-number",
    (v) => {
      project.startNumber = v;
      renderEditor();
    },
    1,
    9999,
    true,
  );
  (document.querySelector("#import-source") as HTMLButtonElement).onclick =
    () => document.querySelector<HTMLInputElement>("#source-input")!.click();
  (document.querySelector("#source-input") as HTMLInputElement).onchange = (
    ev,
  ) => {
    void importFiles((ev.target as HTMLInputElement).files);
  };
  (document.querySelector("#save-project") as HTMLButtonElement).onclick =
    async () => {
      const button =
        document.querySelector<HTMLButtonElement>("#save-project")!;
      button.disabled = true;
      try {
        parseProject(JSON.stringify(project));
        if (invoke) {
          const path = await invoke<string | null>("save_project_to_file", {
            project,
          });
          if (path) notify("프로젝트 JSON을 저장했어요: " + path);
          else notify("프로젝트 저장을 취소했어요.");
        } else {
          download(
            new Blob([JSON.stringify(project, null, 2)], {
              type: "application/json",
            }),
            name("json"),
          );
          notify(
            "프로젝트 JSON 다운로드를 시작했어요. 편집 내용과 문항에 담은 그림이 포함됩니다.",
          );
        }
      } catch (err) {
        notify("프로젝트 저장 실패: " + String(err), true);
      } finally {
        button.disabled = false;
      }
    };
  (document.querySelector("#load-project") as HTMLButtonElement).onclick = () =>
    document.querySelector<HTMLInputElement>("#project-input")!.click();
  (document.querySelector("#project-input") as HTMLInputElement).onchange =
    async (ev) => {
      const f = (ev.target as HTMLInputElement).files?.[0];
      if (!f) return;
      try {
        if (f.size > LIMITS.projectBytes)
          throw new Error("프로젝트 파일은 80MB 이하여야 합니다.");
        const p = parseProject(await f.text());
        if (
          project.items.length &&
          !confirm("현재 편집 내용을 불러온 프로젝트로 바꿀까요?")
        )
          return;
        project = p;
        selected = p.items[0]?.id ?? "";
        pages = [];
        sourceName = "";
        selection = null;
        tab = "preview";
        shell();
        update();
        notify(
          "프로젝트를 불러왔어요. 원본 전체가 필요하면 다시 불러와 주세요.",
        );
      } catch (err) {
        notify(String(err instanceof Error ? err.message : err), true);
      }
    };
  (document.querySelector("#export") as HTMLButtonElement).onclick =
    async () => {
      if (!invoke) {
        notify(
          "HWPX 내보내기는 Rust 엔진이 있는 데스크톱 앱에서 지원합니다. JSON으로 저장해 데스크톱 앱에서 열어 주세요.",
          true,
        );
        return;
      }
      if (!project.items.length) {
        notify("먼저 내보낼 문항을 추가해 주세요.", true);
        return;
      }
      const button = document.querySelector<HTMLButtonElement>("#export")!;
      button.disabled = true;
      notify("한글 문서를 만들고 있어요…");
      try {
        parseProject(JSON.stringify(project));
        validateExportProject(project);
        const path = await invoke<string | null>("export_hwpx_to_file", {
          project,
        });
        if (path)
          notify(
            "HWPX를 저장했어요. 한글에서 수식과 페이지 나눔을 최종 확인해 주세요. 저장 위치: " +
              path,
          );
        else notify("HWPX 내보내기를 취소했어요.");
      } catch (err) {
        notify(
          "내보내기 실패: " + String(err instanceof Error ? err.message : err),
          true,
        );
      } finally {
        button.disabled = false;
      }
    };
}
shell();
if (invoke)
  invoke<typeof runtime>("runtime_status")
    .then((s) => {
      runtime = s;
      showRuntime();
    })
    .catch((err) => notify("로컬 도구 상태 확인 실패: " + String(err), true));
