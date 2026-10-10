import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { JSDOM } from "jsdom";
const asset = readdirSync(new URL("../dist/assets/", import.meta.url)).find(
  (x) => /^index-.*\.js$/.test(x),
);
const code = readFileSync(
  new URL("../dist/assets/" + asset, import.meta.url),
  "utf8",
);
function setup(invoke, saved) {
  const dom = new JSDOM('<!doctype html><div id="app"></div>', {
    url: "http://localhost/",
    runScripts: "outside-only",
  });
  const w = dom.window;
  w.confirm = () => true;
  if (saved) w.localStorage.setItem("munhang-project-v1", saved);
  if (invoke) w.__TAURI__ = { core: { invoke } };
  w.eval(code);
  return {
    dom,
    w,
    d: w.document,
    click: (id) => w.document.querySelector(id).click(),
  };
}
const tick = () => new Promise((r) => setTimeout(r, 0));
test("empty project has useful source and editor states", () => {
  const { d, dom } = setup();
  assert.match(d.body.textContent, /문항의 시작은 원본에서/);
  assert.match(d.body.textContent, /문항을 선택해 주세요/);
  assert.equal(d.querySelector("#count").textContent, "0");
  dom.window.close();
});
test("demo loads two editable questions and preview", () => {
  const { d, click, dom } = setup();
  click("#demo");
  assert.equal(d.querySelectorAll(".question-card").length, 2);
  assert.equal(d.querySelectorAll(".exam-question").length, 2);
  assert.equal(d.querySelector("#total").textContent, "9");
  assert.match(d.querySelector(".exam-sheet").textContent, /함수와 도형/);
  dom.window.close();
});
test("add, edit, reorder, delete and layout update state", () => {
  const { w, d, click, dom } = setup();
  click("#demo");
  click("#add-question");
  assert.equal(d.querySelector("#count").textContent, "3");
  const text = d.querySelector("[data-text]");
  text.value = "<script>alert(1)</script>새 문항";
  text.dispatchEvent(new w.Event("input"));
  assert.ok(!d.querySelector(".exam-sheet script"));
  click("#move-up");
  assert.match(d.querySelectorAll(".question-card")[1].textContent, /새 문항/);
  const columns = d.querySelector("#columns");
  columns.value = "1";
  columns.dispatchEvent(new w.Event("change"));
  assert.ok(d.querySelector(".cols-1"));
  click("#delete-question");
  assert.equal(d.querySelector("#count").textContent, "2");
  assert.equal(
    JSON.parse(w.localStorage.getItem("munhang-project-v1")).items.length,
    2,
  );
  dom.window.close();
});
test("browser export is blocked with honest feedback", () => {
  const { d, click, dom } = setup();
  click("#demo");
  click("#export");
  assert.match(
    d.querySelector("#notice").textContent,
    /Rust 엔진이 있는 데스크톱 앱/,
  );
  dom.window.close();
});
test("native export cancellation does not claim success", async () => {
  const calls = [];
  const { d, click, dom } = setup(async (command, args) => {
    calls.push([command, args]);
    return command === "runtime_status"
      ? { pdf: true, ocr: true, korean: true, formula: false }
      : null;
  });
  click("#demo");
  click("#export");
  await tick();
  assert.match(d.querySelector("#notice").textContent, /취소/);
  assert.ok(
    calls.find(
      (x) => x[0] === "export_hwpx_to_file" && x[1].project.items.length === 2,
    ),
  );
  assert.equal(d.querySelector("#export").disabled, false);
  dom.window.close();
});
test("native JSON save uses backend and reports failures", async () => {
  const { d, click, dom } = setup(async (command) => {
    if (command === "save_project_to_file") throw new Error("disk full");
    return { pdf: false, ocr: false, korean: false, formula: false };
  });
  click("#save-project");
  await tick();
  assert.match(d.querySelector("#notice").textContent, /프로젝트 저장 실패/);
  dom.window.close();
});
test("quota error leaves JSON save route visible", () => {
  const { w, d, click, dom } = setup();
  Object.getPrototypeOf(w.localStorage).setItem = () => {
    throw new Error("QuotaExceededError");
  };
  click("#add-question");
  assert.match(d.querySelector("#save-state").textContent, /자동 저장 실패/);
  assert.ok(d.querySelector("#save-project"));
  dom.window.close();
});

// This exercises orchestration only. Canvas and image decoding are mocked;
// real image fidelity and pointer layout still require a desktop smoke test.
async function loadMockImage(ctx) {
  const { w, d } = ctx;
  w.Image = class {
    naturalWidth = 100;
    naturalHeight = 100;
    set src(value) {
      queueMicrotask(() => this.onload?.());
    }
  };
  w.HTMLCanvasElement.prototype.getContext = () => ({ drawImage() {} });
  w.HTMLCanvasElement.prototype.toDataURL = () => "data:image/png;base64,AQID";
  w.HTMLCanvasElement.prototype.setPointerCapture = () => {};
  w.HTMLCanvasElement.prototype.getBoundingClientRect = () => ({
    left: 0,
    top: 0,
    width: 100,
    height: 100,
  });
  const input = d.querySelector("#source-input");
  Object.defineProperty(input, "files", {
    value: [new w.File(["image"], "test.png", { type: "image/png" })],
  });
  input.dispatchEvent(new w.Event("change"));
  for (let i = 0; i < 8 && !d.querySelector("#source-canvas"); i++)
    await new Promise((r) => setTimeout(r, 10));
  const canvas = d.querySelector("#source-canvas");
  assert.ok(canvas);
  for (const [type, x, y] of [
    ["pointerdown", 10, 10],
    ["pointermove", 80, 80],
    ["pointerup", 80, 80],
  ]) {
    const ev = new w.Event(type);
    Object.assign(ev, { button: 0, pointerId: 1, clientX: x, clientY: y });
    canvas["on" + type](ev);
  }
  assert.equal(d.querySelector("#crop-text").disabled, false);
}
test("source selection adds a crop image block", async () => {
  const ctx = setup();
  ctx.click("#add-question");
  await loadMockImage(ctx);
  ctx.click("#crop-image");
  assert.equal(ctx.d.querySelectorAll(".block-image").length, 1);
  assert.match(ctx.d.querySelector("#notice").textContent, /원본 해상도/);
  ctx.dom.window.close();
});
test("OCR results are rejected when edited target changes during await", async () => {
  let finish;
  const ctx = setup((cmd) =>
    cmd === "runtime_status"
      ? Promise.resolve({ pdf: true, ocr: true, korean: true, formula: false })
      : new Promise((r) => (finish = r)),
  );
  ctx.click("#demo");
  await tick();
  await loadMockImage(ctx);
  ctx.click("#crop-text");
  assert.equal(ctx.d.querySelector("#crop-text").disabled, true);
  ctx.d.querySelectorAll(".question-card")[1].click();
  finish("This must not be inserted into another item");
  await tick();
  assert.match(
    ctx.d.querySelector("#notice").textContent,
    /편집 대상이 바뀌어/,
  );
  assert.ok(
    !ctx.d.querySelector("#item-editor").textContent.includes("This must not"),
  );
  ctx.dom.window.close();
});
test("formula OCR displays source, script and conversion warnings", async () => {
  const ctx = setup(async (cmd) =>
    cmd === "runtime_status"
      ? { pdf: true, ocr: true, korean: true, formula: true }
      : {
          latex: "\\frac{x}{2}",
          script: "{x} over {2}",
          warnings: ["원문 확인 필요"],
        },
  );
  ctx.click("#add-question");
  await tick();
  await loadMockImage(ctx);
  ctx.click("#crop-formula");
  await tick();
  assert.equal(ctx.d.querySelector("[data-equation]").value, "{x} over {2}");
  assert.match(
    ctx.d.querySelector(".formula-evidence").textContent,
    /원문 확인 필요/,
  );
  const saved = ctx.w.localStorage.getItem("munhang-project-v1");
  const blocks = JSON.parse(saved).items[0].blocks;
  assert.equal(blocks.filter((b) => b.kind === "image").length, 0);
  assert.equal(
    blocks.find((b) => b.kind === "equation").sourceLatex,
    "\\frac{x}{2}",
  );
  ctx.click("#tab-preview");
  assert.equal(ctx.d.querySelectorAll(".exam-sheet img").length, 0);
  const restored = setup(undefined, saved);
  assert.match(
    restored.d.querySelector(".formula-evidence").textContent,
    /원문 확인 필요/,
  );
  assert.ok(restored.d.querySelector(".equation-reference-image"));
  restored.dom.window.close();
  ctx.dom.window.close();
});

test("manual equation references remain editable drafts and block HWPX only", async () => {
  const calls = [];
  const ctx = setup(async (cmd) => {
    calls.push(cmd);
    return cmd === "runtime_status"
      ? { pdf: true, ocr: true, korean: true, formula: true }
      : "saved.json";
  });
  ctx.click("#add-question");
  await tick();
  await loadMockImage(ctx);
  ctx.click("#crop-equation");
  assert.ok(ctx.d.querySelector(".equation-reference-image"));
  assert.equal(ctx.d.querySelectorAll(".block-image").length, 0);
  ctx.click("#export");
  await tick();
  assert.match(ctx.d.querySelector("#notice").textContent, /빈 수식/);
  assert.ok(!calls.includes("export_hwpx_to_file"));
  ctx.click("#save-project");
  await tick();
  assert.ok(calls.includes("save_project_to_file"));
  const field = ctx.d.querySelector("[data-equation]");
  field.value = "x^2";
  field.dispatchEvent(new ctx.w.Event("input"));
  assert.equal(ctx.d.querySelector(".equation-required").hidden, true);
  ctx.dom.window.close();
});

function boundaryProject(itemCount = 1, blockCount = 1) {
  return {
    version: 1,
    title: "boundary",
    columns: 2,
    startNumber: 1,
    items: Array.from({ length: itemCount }, (_, i) => ({
      id: "id-" + i,
      points: 4,
      answerSpaceMm: 25,
      blocks: Array.from({ length: blockCount }, () => ({
        kind: "text",
        text: "original",
      })),
    })),
  };
}
test("100-block boundary refuses additions without changing persisted project", () => {
  const original = JSON.stringify(boundaryProject(1, 100));
  const ctx = setup(undefined, original);
  ctx.click("#add-text");
  assert.equal(ctx.d.querySelectorAll(".block").length, 100);
  assert.match(ctx.d.querySelector("#notice").textContent, /100개/);
  assert.equal(ctx.w.localStorage.getItem("munhang-project-v1"), original);
  ctx.dom.window.close();
});
test("500-item boundary refuses additions without changing persisted project", () => {
  const original = JSON.stringify(boundaryProject(500, 1));
  const ctx = setup(undefined, original);
  ctx.click("#add-question");
  assert.equal(ctx.d.querySelectorAll(".question-card").length, 500);
  assert.match(ctx.d.querySelector("#notice").textContent, /500개/);
  assert.equal(ctx.w.localStorage.getItem("munhang-project-v1"), original);
  ctx.dom.window.close();
});
test("oversized text, equation and title input retain prior valid content", () => {
  const ctx = setup();
  ctx.click("#demo");
  for (const [selector, length, property] of [
    ["[data-text]", 200001, "text"],
    ["[data-equation]", 20001, "script"],
    ["#title", 501, "title"],
  ]) {
    const input = ctx.d.querySelector(selector),
      previous = input.value;
    input.value = "x".repeat(length);
    input.dispatchEvent(new ctx.w.Event("input"));
    assert.equal(input.value, previous);
    assert.match(ctx.d.querySelector("#notice").textContent, /까지/);
  }
  const stored = JSON.parse(ctx.w.localStorage.getItem("munhang-project-v1"));
  assert.ok(stored.items[0].blocks[0].text.length < 200001);
  assert.ok(stored.items[0].blocks[1].script.length < 20001);
  ctx.dom.window.close();
});
test("invalid saved JSON stays untouched and exact raw data can be recovered", async () => {
  const original = "{bad json: <script>recovery</script>\n";
  let exported;
  const ctx = setup(async (cmd, args) => {
    if (cmd === "save_recovery_text_to_file") {
      exported = args.text;
      return "recovery.json";
    }
    return { pdf: false, ocr: false, korean: false, formula: false };
  }, original);
  assert.ok(ctx.d.querySelector(".recovery-panel"));
  ctx.click("#show-recovery");
  assert.equal(ctx.d.querySelector("#recovery-raw").value, original);
  assert.equal(ctx.d.querySelectorAll("script").length, 0);
  ctx.click("#add-question");
  ctx.click("#add-text");
  assert.equal(ctx.w.localStorage.getItem("munhang-project-v1"), original);
  assert.match(
    ctx.d.querySelector("#save-state").textContent,
    /자동 저장 중지/,
  );
  ctx.click("#download-recovery");
  await tick();
  assert.equal(exported, original);
  assert.equal(ctx.w.localStorage.getItem("munhang-project-v1"), original);
  ctx.dom.window.close();
});
test("oversized OCR output is rejected without changing prior content", async () => {
  const ctx = setup(async (cmd) =>
    cmd === "runtime_status"
      ? { pdf: true, ocr: true, korean: true, formula: false }
      : "x".repeat(200001),
  );
  ctx.click("#add-question");
  await tick();
  await loadMockImage(ctx);
  const original = ctx.w.localStorage.getItem("munhang-project-v1");
  ctx.click("#crop-text");
  await tick();
  assert.equal(ctx.w.localStorage.getItem("munhang-project-v1"), original);
  assert.match(
    ctx.d.querySelector("#notice").textContent,
    /추가하지 않았습니다/,
  );
  ctx.dom.window.close();
});
