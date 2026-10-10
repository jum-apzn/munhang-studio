export const LIMITS = {
  items: 500,
  blocks: 100,
  text: 200000,
  script: 20000,
  title: 500,
  projectBytes: 80 * 1024 * 1024,
  exportScriptBytes: 16000,
} as const;
export type Block =
  | { kind: "text"; text: string }
  | {
      kind: "equation";
      script: string;
      sourceImage?: string;
      sourceLatex?: string;
      conversionWarnings?: string[];
    }
  | { kind: "image"; dataUrl: string; widthMm: number; alt: string };
export interface Item {
  id: string;
  points: number;
  answerSpaceMm: number;
  blocks: Block[];
}
export interface Project {
  version: 1;
  title: string;
  columns: 1 | 2;
  startNumber: number;
  items: Item[];
}
export interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}
export const blankProject = (): Project => ({
  version: 1,
  title: "나의 수학 문제지",
  columns: 2,
  startNumber: 1,
  items: [],
});
export function normalizedRect(
  x1: number,
  y1: number,
  x2: number,
  y2: number,
  width: number,
  height: number,
): Rect {
  const clamp = (v: number, max: number) => Math.max(0, Math.min(max, v));
  const left = clamp(Math.min(x1, x2), width),
    top = clamp(Math.min(y1, y2), height);
  return {
    x: left,
    y: top,
    width: clamp(Math.max(x1, x2), width) - left,
    height: clamp(Math.max(y1, y2), height) - top,
  };
}
export function reorder<T>(items: T[], from: number, to: number): T[] {
  const next = [...items];
  if (from < 0 || to < 0 || from >= items.length || to >= items.length)
    return next;
  const [item] = next.splice(from, 1);
  next.splice(to, 0, item);
  return next;
}
const record = (v: unknown): v is Record<string, unknown> =>
  typeof v === "object" && v !== null && !Array.isArray(v);
const number = (v: unknown, min: number, max: number) =>
  typeof v === "number" && Number.isFinite(v) && v >= min && v <= max;
const imageDataUrl = (value: unknown): value is string =>
  typeof value === "string" &&
  /^data:image\/(png|jpeg);base64,[A-Za-z0-9+/=\r\n]+$/.test(value);
export function assertUtf8Limit(
  value: string,
  maxBytes: number,
  label: string,
): void {
  if (new Blob([value]).size > maxBytes)
    throw new Error(
      `${label} 크기 제한(${maxBytes.toLocaleString()} bytes)을 초과했습니다. 기존 내용을 유지합니다.`,
    );
}
export function parseProject(input: string): Project {
  assertUtf8Limit(input, LIMITS.projectBytes, "프로젝트 JSON (80 MiB)");
  const p: unknown = JSON.parse(input);
  if (
    !record(p) ||
    p.version !== 1 ||
    typeof p.title !== "string" ||
    p.title.length > LIMITS.title ||
    (p.columns !== 1 && p.columns !== 2) ||
    !number(p.startNumber, 1, 9999) ||
    !Number.isInteger(p.startNumber) ||
    !Array.isArray(p.items) ||
    p.items.length > LIMITS.items
  )
    throw new Error(
      "지원하는 문항 프로젝트 파일이 아닙니다. 형식과 버전을 확인해 주세요.",
    );
  const ids = new Set<string>();
  for (const item of p.items) {
    if (
      !record(item) ||
      typeof item.id !== "string" ||
      ids.has(item.id) ||
      !number(item.points, 0, 1000) ||
      !number(item.answerSpaceMm, 0, 200) ||
      !Array.isArray(item.blocks) ||
      item.blocks.length > LIMITS.blocks
    )
      throw new Error("문항 정보가 올바르지 않습니다.");
    ids.add(item.id);
    for (const b of item.blocks) {
      if (!record(b)) throw new Error("내용 블록 형식이 올바르지 않습니다.");
      if (
        b.kind === "text" &&
        typeof b.text === "string" &&
        b.text.length <= LIMITS.text
      )
        continue;
      if (
        b.kind === "equation" &&
        typeof b.script === "string" &&
        b.script.length <= LIMITS.script &&
        (b.sourceImage === undefined || imageDataUrl(b.sourceImage)) &&
        (b.sourceLatex === undefined ||
          (typeof b.sourceLatex === "string" &&
            b.sourceLatex.length <= 20000)) &&
        (b.conversionWarnings === undefined ||
          (Array.isArray(b.conversionWarnings) &&
            b.conversionWarnings.length <= 100 &&
            b.conversionWarnings.every(
              (w) => typeof w === "string" && w.length <= 2000,
            )))
      )
        continue;
      if (
        b.kind === "image" &&
        typeof b.dataUrl === "string" &&
        /^data:image\/(png|jpeg);base64,[A-Za-z0-9+/=\r\n]+$/.test(b.dataUrl) &&
        number(b.widthMm, 1, 180) &&
        typeof b.alt === "string"
      )
        continue;
      throw new Error("지원하지 않거나 손상된 내용 블록이 있습니다.");
    }
  }
  assertUtf8Limit(
    JSON.stringify(p, null, 2),
    LIMITS.projectBytes,
    "저장할 프로젝트 JSON (80 MiB)",
  );
  return p as unknown as Project;
}
export const demoProject = (): Project => ({
  version: 1,
  title: "함수와 도형 · 개념 확인",
  columns: 2,
  startNumber: 1,
  items: [
    {
      id: "demo-1",
      points: 4,
      answerSpaceMm: 25,
      blocks: [
        {
          kind: "text",
          text: "이차함수 f(x) = x² − 4x + 3의 그래프가 x축과 만나는 두 점 사이의 거리를 구하시오.",
        },
        { kind: "equation", script: "f(x) = x^2 - 4x + 3" },
      ],
    },
    {
      id: "demo-2",
      points: 5,
      answerSpaceMm: 35,
      blocks: [
        {
          kind: "text",
          text: "반지름의 길이가 3인 원의 넓이를 구하시오. 풀이 과정을 함께 쓰시오.",
        },
        { kind: "equation", script: "S = pi r^2" },
      ],
    },
  ],
});
export function dataUrlBytes(url: string): number[] {
  const s = atob(url.split(",")[1]);
  return Array.from(s, (c) => c.charCodeAt(0));
}
export function escapeHtml(value: unknown): string {
  return String(value).replace(
    /[&<>"']/g,
    (c) =>
      ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[
        c
      ]!,
  );
}

export function validateExportProject(project: Project): void {
  for (const [i, item] of project.items.entries()) {
    for (const block of item.blocks) {
      if (block.kind === "equation")
        assertUtf8Limit(
          block.script,
          LIMITS.exportScriptBytes,
          `문항 ${project.startNumber + i} 수식 내보내기 (긴 수식은 JSON 초안으로 보관 가능)`,
        );
    }
    if (
      item.blocks.some(
        (block) => block.kind === "equation" && !block.script.trim(),
      )
    )
      throw new Error(
        `문항 ${project.startNumber + i}에 빈 수식이 있습니다. 원본을 참고해 한컴 수식 스크립트를 입력하거나 빈 블록을 삭제해 주세요. JSON에는 초안으로 저장할 수 있습니다.`,
      );
  }
}
