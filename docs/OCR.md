# Local PDF and OCR runtime

## What is actually implemented

`crates/core/src/runtime.rs` runs locally installed Poppler `pdftoppm` and Tesseract, directly with argument arrays. No shell evaluation, document upload, account, paid API, or runtime model download occurs. The UI must expose `runtime_status()` and must not imply Korean support when `korean` is false. English-only fallback is usable for English, not Korean. OCR is a draft requiring review; poor scans, tables, reading order and handwriting can fail.

- `import_pdf(&[u8])`: PDF to PNG data URLs, up to 20 pages; longer PDFs are rejected rather than silently truncated. 160 DPI, longest side at most 2200 pixels. Importing a PDF does not automatically recover editable paragraphs or formulas.
- `recognize_image(&[u8])`: plain text from a selected PNG/JPEG image crop, `kor+eng` when available, otherwise `kor` or `eng`. Tesseract PSM 6 assumes one text region. Crop individual question text for best results.
- `recognize_formula(&[u8])`: optional, separately configured PP-FormulaNet_plus-S CPU sidecar returns raw LaTeX from one formula crop. It has a 120-second timeout. The app may assist limited LaTeX-to-Hancom conversion, but every OCR/conversion result needs review; unsupported constructs must be flagged. Tesseract itself does not recognize editable equation objects.
- `runtime_status().formula` reports whether explicit local Python/model paths are configured and required model files exist. It does not guarantee every runtime dependency or every formula will work; initialization/inference errors are shown when recognition runs.
- 40 MiB input cap; 25-million-pixel image cap; decoded image allocation limit; 90-second PDF / 60-second OCR child timeout; bounded stdout/stderr retention; 180 MiB monitored temporary output budget; one import/OCR job at a time. Temporary directories are unique and removed on normal success/error. Abrupt application termination can leave OS temp files. These limits mitigate accidental overload, but subprocesses are not an OS security sandbox and have no hard process memory quota. Use maintained system-tool versions for untrusted files.

## Install dependencies

macOS, using the official Homebrew registry:

```sh
brew install poppler tesseract tesseract-lang
```

Debian/Ubuntu distribution repositories:

```sh
sudo apt-get update
sudo apt-get install poppler-utils tesseract-ocr tesseract-ocr-kor tesseract-ocr-eng
```

Windows: Tesseract upstream links available builds in its [installation guide](https://tesseract-ocr.github.io/tessdoc/Installation.html). This project does not silently install third-party Windows executables. Install a trusted Tesseract build and Poppler distribution, put their executable directories on PATH, and verify `tesseract --list-langs` and `pdftoppm -v`. A Windows installer bundling those engines has not been produced or tested here. Linux package commands cannot substitute for Windows installation.

Alternatively, after Tesseract is installed, download the official Korean trained data to a user-owned folder:

```sh
bash scripts/setup-ocr.sh --download-korean
# Follow the printed TESSDATA_PREFIX export before launching the app.
```

```powershell
.\scripts\setup-ocr.ps1 -DownloadKorean
# Launch the application in this same PowerShell session.
```

The scripts pin official tessdata_fast commit `87416418657359cb625c412a48b6e1d6d41c29bd` and verify SHA-256 `6b85e11d9bbf07863b97b3523b1b112844c43e713df8b66418a081fd1060b3b2` before installing `kor.traineddata`. A custom `TESSDATA_PREFIX` replaces the normal data search path: also copy your installed `eng.traineddata` there if bilingual OCR is wanted. The scripts do not change global settings or install binaries. Without a download flag, the shell script only checks installed tools.

Launch environment matters: desktop shortcuts may not inherit terminal PATH/TESSDATA_PREFIX. Restart the application after installing engines/models. App status reflects its own process environment.

## Validation performed

On the Linux build environment, actual Tesseract recognition was run against generated, clean raster fixtures:

- English: `Mathematics worksheet 123`, recognized exactly.
- Korean: `수학 문제를 풀고 정답을 쓰시오.` and `삼각형의 넓이를 구하시오. 123`, recognized exactly with official `tessdata_fast` Korean weights plus installed English weights.

These are smoke tests, not an accuracy benchmark on real worksheets. The Korean model was placed outside the repository and selected with TESSDATA_PREFIX. It is not automatically installed on another user's computer. A clean printed formula fixture was also tested with the optional Paddle sidecar; no real-worksheet or handwriting accuracy benchmark was run.

## Optional Paddle formula sidecar: implemented, explicit setup required

The Rust adapter embeds `scripts/setup-ocr.py` and invokes it with absolute configured Python/model paths. Inference uses `model_dir`, CPU, one local crop and offline flags; per-request `PADDLE_PDX_CACHE_HOME` and `PADDLE_EXTENSION_DIR` keep caches in the temporary directory without changing HOME; no document upload, runtime model download, or service account. First-time setup must be done separately. This optional Python/Paddle runtime is not bundled into the desktop installer. Actual Windows/macOS native runtime and packaging remain untested.

Tested Linux x86-64 / Python 3.12 setup (official Paddle CPU index and PyPI):

```sh
python -m venv .venv-paddle
.venv-paddle/bin/python -m pip install paddlepaddle==3.0.0 -i https://www.paddlepaddle.org.cn/packages/stable/cpu/
.venv-paddle/bin/python -m pip install paddleocr==3.7.0 paddlex==3.7.2 tokenizers==0.23.3 ftfy==6.3.1
# On a SOCKS-proxy environment, also install socksio if needed for model setup.
.venv-paddle/bin/python scripts/setup-ocr.py --download-model-dir "$HOME/.local/share/munhang-studio/formula-model"
export MUNHANG_FORMULA_PYTHON="$(pwd)/.venv-paddle/bin/python"
export MUNHANG_FORMULA_MODEL_DIR="$HOME/.local/share/munhang-studio/formula-model"
# Launch the desktop app from this terminal.
```

On Windows use the venv's `Scripts/python.exe` and set the two environment variables to absolute paths in PowerShell. A compatibility-tested Windows dependency lock or installer is not yet supplied. Setup requires `curl` for downloads. This download mode fetches only official PaddlePaddle files from Hugging Face, pins model revision `3d46f557e3a1752f4bf81202395af3b5ecfadfd2`, and checks SHA-256 for config and weights before replacing local files. Existing valid model files are reused. The original model card is kept as `MODEL_CARD.md` with Apache-2.0 metadata. A source checkout does not include the 257 MB parameter file.

Manual sidecar check after setup:

```sh
"$MUNHANG_FORMULA_PYTHON" scripts/setup-ocr.py --model-dir "$MUNHANG_FORMULA_MODEL_DIR" --input formula-crop.png
```

### Measured formula smoke test

Input: a generated raster of `x² + 2x + 1 = (x+1)²` using Matplotlib mathtext.
Expected/actual LaTeX: `x^{2}+2x+1=(x+1)^{2}` (exact match).
Observed fresh-process startup plus inference: 6.42 seconds, peak resident memory about 806 MiB, CPU-only, MKL-DNN disabled, two Paddle CPU threads. This is one clean printed fixture, not a general accuracy claim. Model loading happens on each request in this MVP, trading speed for simple isolation. First install revealed missing tokenizers/ftfy dependencies; these are included above. Korean body text continues to use tested Tesseract; the Korean Paddle recognizer remains researched rather than integrated.

Re-run Rust engine integration tests with locally prepared fixtures:

```sh
# Fixtures: english.png, korean.png, one-page.pdf, too-many.pdf (21 pages), formula.png
export MUNHANG_FIXTURES=/absolute/path/to/fixtures
export TESSDATA_PREFIX=/absolute/path/to/tessdata-containing-kor-and-eng
cargo test -p munhang-core local_engine_smoke -- --ignored --nocapture
cargo test -p munhang-core local_formula_smoke -- --ignored --nocapture
```

### Candidate models and official sources

- [korean_PP-OCRv5_mobile_rec model card](https://huggingface.co/PaddlePaddle/korean_PP-OCRv5_mobile_rec/blob/main/README.md): published by PaddlePaddle, tagged Apache-2.0. This is a text-line recognizer; full-page OCR also needs text detection and reading-order handling. [Official files](https://huggingface.co/PaddlePaddle/korean_PP-OCRv5_mobile_rec/tree/main).
- [PP-FormulaNet_plus-S model card](https://huggingface.co/PaddlePaddle/PP-FormulaNet_plus-S/blob/main/README.md): published by PaddlePaddle, tagged Apache-2.0. Produces LaTeX, not Hancom script, and focuses on English mathematical formulas. [Official files](https://huggingface.co/PaddlePaddle/PP-FormulaNet_plus-S/tree/main).
- [Official formula pipeline benchmarks](https://paddlepaddle.github.io/PaddleOCR/main/en/version3.x/pipeline_usage/formula_recognition.html) list plus-S at 248 MB and roughly 261 ms in the reported high-performance CPU test; ordinary CPU mode is not reported there. This is not a promise for a user's laptop, Korean handwriting, or full-page parsing. Model memory use exceeds its file size.

The Korean text model is a future alternative to Tesseract; its Python `TextRecognition` API is documented upstream but not used by this app. Formula output is LaTeX. Preserve it alongside the crop and review the limited Hancom conversion; unsupported constructs must never be silently asserted correct.

## License / redistribution notes

- Tesseract and tessdata_fast: Apache-2.0. [Tesseract license](https://github.com/tesseract-ocr/tesseract/blob/main/LICENSE), [trained-data license](https://github.com/tesseract-ocr/tessdata_fast/blob/main/LICENSE).
- PaddleOCR code: [Apache-2.0](https://github.com/PaddlePaddle/PaddleOCR/blob/main/LICENSE); both candidate model cards explicitly tag Apache-2.0. Retain required notices/licenses when distributing code or weights and audit transitive dependencies separately.
- Poppler is GPL-licensed; consult the [upstream site](https://poppler.freedesktop.org/) and package copyright files before bundling it. This source project currently invokes a separately installed system tool; it does not redistribute a Poppler binary. Do not describe a future all-in-one installer as wholly Apache-2.0 without a dependency/license review.

Official documentation/model cards checked 2026-10-10. No paid service credentials are needed for this local fallback or the documented public-model experiment.
