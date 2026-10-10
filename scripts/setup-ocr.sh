#!/usr/bin/env bash
# No sudo, packages, paid APIs, or document upload. Optional official model download.
set -euo pipefail
DEST="${MUNHANG_TESSDATA_DIR:-${HOME}/.local/share/munhang-studio/tessdata}"
REV=87416418657359cb625c412a48b6e1d6d41c29bd
HASH=6b85e11d9bbf07863b97b3523b1b112844c43e713df8b66418a081fd1060b3b2
if [[ "${1:-}" == "--download-korean" ]]; then
  mkdir -p "$DEST"
  TMP=$(mktemp -d "$DEST/.download-XXXXXX")
  trap 'rm -rf "$TMP"' EXIT
  curl --fail --location --proto '=https' --tlsv1.2 --max-time 180 \
    "https://raw.githubusercontent.com/tesseract-ocr/tessdata_fast/$REV/kor.traineddata" -o "$TMP/kor.traineddata"
  if command -v sha256sum >/dev/null 2>&1; then
    ACTUAL=$(sha256sum "$TMP/kor.traineddata" | cut -d' ' -f1)
  else
    ACTUAL=$(shasum -a 256 "$TMP/kor.traineddata" | cut -d' ' -f1)
  fi
  [[ "$ACTUAL" == "$HASH" ]] || { echo 'Checksum mismatch; model not installed.' >&2; exit 1; }
  curl --fail --location --proto '=https' --tlsv1.2 --max-time 60 \
    "https://raw.githubusercontent.com/tesseract-ocr/tessdata_fast/$REV/LICENSE" -o "$TMP/LICENSE-tessdata_fast.txt"
  mv "$TMP/kor.traineddata" "$DEST/kor.traineddata"
  mv "$TMP/LICENSE-tessdata_fast.txt" "$DEST/LICENSE-tessdata_fast.txt"
  echo "Korean Apache-2.0 data installed in: $DEST"
  printf 'Launch from a terminal with: export TESSDATA_PREFIX=%q\n' "$DEST"
  echo 'This override contains Korean only. Copy an existing eng.traineddata into this directory for kor+eng.'
elif [[ -n "${1:-}" ]]; then
  echo 'Usage: bash scripts/setup-ocr.sh [--download-korean]' >&2; exit 2
fi
if command -v pdftoppm >/dev/null 2>&1; then pdftoppm -v; else echo 'Missing pdftoppm. See docs/OCR.md.'; fi
if command -v tesseract >/dev/null 2>&1; then
  if [[ "${1:-}" == "--download-korean" ]]; then TESSDATA_PREFIX="$DEST" tesseract --list-langs; else tesseract --list-langs; fi
else echo 'Missing Tesseract. See docs/OCR.md.'; fi
