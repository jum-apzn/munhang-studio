# Frontend verification

## Automated checks

Run `npm ci`, then `npm test`. Development server: `npm run dev` on port 1420. Node.js 22.13+ or 24 is needed for the TypeScript test runner. Production output is `dist/`, built by `npm run build`.

Verified in this workspace on 2026-10-10:

- TypeScript strict checking and Vite production build: passed.
- 14 pure utility tests: project round trip, empty project, reversed/out-of-bounds rectangle normalization, immutable reorder, duplicate IDs, remote image URL rejection, invalid numbers, HTML escaping, image byte decoding, persisted equation metadata, invalid metadata rejection, and draft-versus-export validation, UTF-8 byte limits, and the 16,000-byte equation export boundary, and an actual over-80-MiB input payload.
- 16 jsdom interaction tests: empty states, synthetic demo, editing/reordering/deletion/layout, honest browser-only export limitation, native export cancellation, native JSON save failure, localStorage quota feedback, mocked image cropping, stale OCR target rejection, formula OCR result/warning persistence through reload, reference-image exclusion from preview, manual equation draft save/export behavior, 100-block and 500-item boundary guards, oversized typing/OCR rejection, and exact invalid-storage recovery without overwrite.

The DOM suite loads the actual Vite production bundle. Tauri commands are mocked. Canvas decoding/drawing and pointer geometry are mocked for crop tests. These tests check application orchestration, not actual OCR quality, native dialog integration, or pixel fidelity.

## Visual and native smoke tests still required

An ordinary Chromium launch failed because this execution environment denies the browser's OS socket creation (`socket() failed: Operation not permitted`). No security restriction was bypassed and no browser screenshot was obtained. **The UI has not been visually validated in a real browser or Tauri WebView here.** This is not evidence of a successful native end-to-end test.

On a supported desktop, check:

1. Start the Tauri app and verify Korean labels at 1440×1000 and 1024×768, plus keyboard focus.
2. Import a PNG/JPEG and a multi-page PDF; navigate pages; drag in every direction and near boundaries.
3. Add a crop as a new question, a figure, and a manual equation reference. Verify source aspect ratio and original crop detail.
4. Run installed local OCR and formula OCR. Compare recognized text, LaTeX, conversion warnings, and Hancom script with the source.
5. Change/delete the selected question during OCR. The result must not attach to another question.
6. Edit and reorder blocks/questions, change points and answer-space millimeters, and switch between 1 and 2 columns.
7. Save a project, reload it, and verify images and edits. Native save cancellation should report cancellation only.
8. Export HWPX with the native dialog, open it in Hancom, and inspect equations, pictures, column layout, and page breaks.

## Intentional boundaries

- Browser preview supports PNG/JPEG and JSON editing. PDF import, OCR, and HWPX generation require the desktop Rust backend; no export is simulated.
- Worksheet preview is approximate. It displays editable Hancom equation scripts rather than claiming to render their final appearance.
- Original pages are temporary workspace sources. Project JSON contains editable questions and their retained crop images, not complete original documents.
- Equation reference images, raw formula OCR LaTeX, and conversion warnings are saved as equation metadata in project JSON and displayed only in the editor. They are not duplicated into the printable preview or HWPX. Empty equation scripts are valid editable drafts for JSON saving, but must be corrected or removed before HWPX export.
- Source files and OCR crops are never uploaded by this frontend. It uses no remote fonts or image assets.
- Local browser persistence is best-effort and can fail when image-heavy projects exceed quota. A visible warning directs the user to JSON saving.

## Persistence and size-limit regression protection

The editor and project parser share limits: 500 questions, 100 blocks per question, 200,000 characters per text block, 20,000 characters per equation script, and 500 title characters. New blocks/questions are validated before mutation; oversized manual input and OCR results leave existing data intact. Automatic and explicit project saves validate the current project before persistence.

If startup cannot parse saved data, a persistent recovery panel appears. The original localStorage entry is preserved, automatic persistence stays disabled, and the exact original JSON can be inspected/copied or saved through `save_recovery_text_to_file` on desktop (a raw download in browser). Editing a new temporary workspace does not overwrite that recovery source. Recovery does not silently normalize, truncate, or discard the original data.

The overall project limit is 80 MiB of UTF-8 JSON, checked both on incoming data and canonical pretty-printed output before persistence/download. Field-edit transactions restore the last valid in-memory state if validation fails. Native save separately validates its final pretty-printed bytes. Equation drafts allow up to 20,000 characters, while HWPX export enforces the core's stricter 16,000 UTF-8-byte limit; larger scripts are retained as drafts rather than truncated.
