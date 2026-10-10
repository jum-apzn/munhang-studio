import test from "node:test";
import assert from "node:assert/strict";
import {
  blankProject,
  LIMITS,
  assertUtf8Limit,
  demoProject,
  parseProject,
  validateExportProject,
  normalizedRect,
  reorder,
  escapeHtml,
  dataUrlBytes,
} from "./utils.ts";
test("project round trip", () =>
  assert.deepEqual(parseProject(JSON.stringify(demoProject())), demoProject()));
test("blank project accepted", () =>
  assert.deepEqual(
    parseProject(JSON.stringify(blankProject())),
    blankProject(),
  ));
test("drag rectangle direction independent and clamped", () =>
  assert.deepEqual(normalizedRect(120, 90, -5, 10, 100, 80), {
    x: 0,
    y: 10,
    width: 100,
    height: 70,
  }));
test("reorder immutable", () => {
  const a = [1, 2, 3];
  assert.deepEqual(reorder(a, 0, 2), [2, 3, 1]);
  assert.deepEqual(a, [1, 2, 3]);
  assert.deepEqual(reorder(a, -1, 0), a);
});
test("duplicate ids rejected", () => {
  const p = demoProject();
  p.items[1].id = p.items[0].id;
  assert.throws(() => parseProject(JSON.stringify(p)));
});
test("hostile image URL rejected", () => {
  const p = demoProject();
  p.items[0].blocks = [
    {
      kind: "image",
      dataUrl: "https://tracker.test/a.png",
      widthMm: 50,
      alt: "",
    },
  ];
  assert.throws(() => parseProject(JSON.stringify(p)));
});
test("invalid numeric values rejected", () => {
  const p = demoProject();
  p.startNumber = 1.5;
  assert.throws(() => parseProject(JSON.stringify(p)));
});
test("HTML escaped", () =>
  assert.equal(escapeHtml('<script>"&'), "&lt;script&gt;&quot;&amp;"));
test("bytes decode", () =>
  assert.deepEqual(dataUrlBytes("data:image/png;base64,AQID"), [1, 2, 3]));

test("equation metadata and empty drafts round trip", () => {
  const p = demoProject();
  p.items[0].blocks = [
    {
      kind: "equation",
      script: "",
      sourceImage: "data:image/png;base64,AQID",
      sourceLatex: "\\unsupported{x}",
      conversionWarnings: ["수동 수정 필요"],
    },
  ];
  assert.deepEqual(parseProject(JSON.stringify(p)), p);
  assert.throws(() => validateExportProject(p), /빈 수식/);
  p.items[0].blocks[0].script = "x";
  assert.doesNotThrow(() => validateExportProject(p));
});
test("equation metadata rejects remote images and malformed warnings", () => {
  const p = demoProject();
  p.items[0].blocks = [
    { kind: "equation", script: "x", sourceImage: "https://example.com/a.png" },
  ];
  assert.throws(() => parseProject(JSON.stringify(p)));
  const json = JSON.parse(JSON.stringify(demoProject()));
  json.items[0].blocks = [
    { kind: "equation", script: "x", conversionWarnings: [123] },
  ];
  assert.throws(() => parseProject(JSON.stringify(json)));
});

test("UTF-8 size checks count bytes and reject overflow without truncation", () => {
  assert.equal(LIMITS.projectBytes, 80 * 1024 * 1024);
  assert.doesNotThrow(() => assertUtf8Limit("가", 3, "test"));
  assert.throws(() => assertUtf8Limit("가", 2, "test"), /크기 제한/);
  assert.doesNotThrow(() => assertUtf8Limit("x".repeat(1024), 1024, "test"));
  assert.throws(() => assertUtf8Limit("x".repeat(1025), 1024, "test"));
});
test("long equations are preserved as drafts but export enforces 16000 UTF8 bytes", () => {
  const p = demoProject();
  p.items[0].blocks = [{ kind: "equation", script: "x".repeat(16001) }];
  assert.deepEqual(parseProject(JSON.stringify(p)), p);
  assert.throws(() => validateExportProject(p), /16,000/);
  p.items[0].blocks = [{ kind: "equation", script: "가".repeat(5334) }];
  assert.throws(() => validateExportProject(p));
  p.items[0].blocks = [{ kind: "equation", script: "x".repeat(16000) }];
  assert.doesNotThrow(() => validateExportProject(p));
});

test("project parser rejects an actual over-80MiB UTF8 payload", () => {
  const oversized = " ".repeat(LIMITS.projectBytes) + "{}";
  assert.throws(() => parseProject(oversized), /80 MiB/);
});
