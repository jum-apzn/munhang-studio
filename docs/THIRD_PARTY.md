# Third-party components and provenance

Application code is original. The working name is not a trademark clearance.

- Rust standard library/toolchain: MIT or Apache-2.0, https://www.rust-lang.org/policies/licenses
- Tauri 2: MIT or Apache-2.0, https://github.com/tauri-apps/tauri
- serde / serde_json: MIT or Apache-2.0, https://github.com/serde-rs/serde
- zip-rs: MIT, https://github.com/zip-rs/zip2
- base64: MIT or Apache-2.0, https://github.com/marshallpierce/rust-base64
- image: MIT or Apache-2.0, https://github.com/image-rs/image
- tempfile: MIT or Apache-2.0, https://github.com/Stebalien/tempfile
- Vite: MIT, https://github.com/vitejs/vite
- TypeScript: Apache-2.0, https://github.com/microsoft/TypeScript
- Tesseract OCR: Apache-2.0. Invoked as an optional separately installed local executable, https://github.com/tesseract-ocr/tesseract
- Poppler pdftoppm: GPL-2.0-or-later. Optional separately installed executable; NOT bundled in this source archive. Users/distributors must observe Poppler license obligations if redistributing binaries. https://poppler.freedesktop.org/

Cargo.lock and package-lock.json identify the resolved dependency versions.
Fonts use the local system font stack and are not distributed by the application.
Synthetic sample questions, diagrams, and project data in examples/ are original and licensed with this project.

HWPX format research and any precise implementation caveats are in HWPX.md. No kordoc source is copied into this project.

Optional formula runtime: PaddlePaddle/PaddleOCR/PaddleX and PP-FormulaNet_plus-S (Apache-2.0 model card), installed separately. See OCR.md for pinned versions, source links and setup. The model and Python environment are not redistributed.
Development-only test tools: jsdom (MIT), Prettier (MIT), python-hwpx 6.8.0 (Apache-2.0). See lockfiles and respective package metadata.
