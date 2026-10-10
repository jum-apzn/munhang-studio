# Original example

All questions and the diagram in sample-project.json were created for this project. No source worksheet or proprietary asset is included. The equation blocks are Hancom equation scripts.

The HWPX is a structural and independent-parser smoke-test artifact. Actual Hancom Office rendering, equation editing and re-save round-trip still require manual validation.

`validation-report.json` records python-hwpx 6.8.0 package, schema-lint and reopen checks plus raw 2024-schema deviations. This does not prove Hancom application compatibility.

formula-ocr-project.json and formula-ocr-worksheet.hwpx were produced by running local PP-FormulaNet_plus-S on fixtures/formula.png, converting the recognized LaTeX with the Rust subset converter, and exporting with the Rust core. Source OCR metadata is retained in JSON but does not print a duplicate equation image. See verification/formula-result.json and verification/formula-hwpx-validation.json.

fixtures/ contains original generated Korean/English text, a quadratic identity, and 1/21-page synthetic PDFs. They contain no personal/user documents. too-many.pdf exercises the 20-page rejection boundary.
