#!/usr/bin/env python3
"""Optional local FormulaNet sidecar; dependencies and weights must be installed first.

Inference performs no downloads. The explicit --download-model-dir setup mode
downloads pinned public weights. Runtime uses an existing local model folder. It prints only raw LaTeX on stdout, diagnostics on stderr. No eval/shell.
"""
import argparse
import contextlib
import os
from pathlib import Path
import sys

os.environ.setdefault("PADDLE_PDX_DISABLE_MODEL_SOURCE_CHECK", "True")
os.environ.setdefault("HF_HUB_OFFLINE", "1")
os.environ.setdefault("PADDLE_PDX_OFFLINE", "1")
os.environ.setdefault("OMP_NUM_THREADS", "1")


def download_model(destination):
    """Explicit setup-only network operation, using pinned official publisher files."""
    import hashlib
    import subprocess
    import tempfile
    revision = "3d46f557e3a1752f4bf81202395af3b5ecfadfd2"
    files = {
        "config.json": "ddf1f951ceeb10c2b9de3ae255b62de59299b1028f87878177e86c9604fe3610",
        "inference.json": "01238434e33df83588e2627f350559b576e34551d2b2ffea148345032de56c00",
        "inference.pdiparams": "e464f94412feaa98f8791eacc84684f887b3569e30e80c52b8112e9cf7d4069b",
        "inference.yml": "96062655d94c21d39274328dbc82c1a487e66addb8425f5a7fd5b7dfb2421ec3",
    }
    folder = Path(destination).expanduser().resolve()
    folder.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="formula-download-", dir=folder) as temporary:
        for filename, expected in files.items():
            target = folder / filename
            if target.is_file() and hashlib.sha256(target.read_bytes()).hexdigest() == expected:
                continue
            candidate = Path(temporary) / filename
            url = f"https://huggingface.co/PaddlePaddle/PP-FormulaNet_plus-S/resolve/{revision}/{filename}"
            subprocess.run(["curl", "--fail", "--location", "--proto", "=https", "--tlsv1.2", "--max-time", "600", url, "-o", str(candidate)], check=True)
            if hashlib.sha256(candidate.read_bytes()).hexdigest() != expected:
                raise RuntimeError(f"Checksum mismatch: {filename}")
            candidate.replace(target)
        subprocess.run(["curl", "--fail", "--location", "--proto", "=https", "--tlsv1.2", "--max-time", "60", f"https://huggingface.co/PaddlePaddle/PP-FormulaNet_plus-S/resolve/{revision}/README.md", "-o", str(folder / "MODEL_CARD.md")], check=True)
    print(f"Pinned Apache-2.0 PP-FormulaNet_plus-S files verified in {folder}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--download-model-dir", help="Explicitly download official pinned weights; setup only")
    parser.add_argument("--model-dir")
    parser.add_argument("--input")
    args = parser.parse_args()
    if args.download_model_dir:
        download_model(args.download_model_dir)
        return
    if not args.model_dir or not args.input:
        parser.error("Use --download-model-dir or both --model-dir and --input")
    model_dir = Path(args.model_dir).resolve(strict=True)
    source = Path(args.input).resolve(strict=True)
    for filename in ("config.json", "inference.json", "inference.pdiparams", "inference.yml"):
        if not (model_dir / filename).is_file():
            raise RuntimeError(f"Local model file missing: {filename}. Pre-install official PP-FormulaNet_plus-S weights.")
    if source.stat().st_size > 40 * 1024 * 1024:
        raise RuntimeError("Image exceeds 40 MiB")
    with contextlib.redirect_stdout(sys.stderr):
        from paddleocr import FormulaRecognition
        model = FormulaRecognition(model_name="PP-FormulaNet_plus-S", model_dir=str(model_dir), device="cpu", enable_mkldnn=False, cpu_threads=2)
        results = list(model.predict(input=str(source), batch_size=1))
    if len(results) != 1:
        raise RuntimeError("Expected exactly one cropped formula result")
    text = results[0].get("rec_formula", "")
    if not isinstance(text, str) or not text.strip() or len(text) > 100_000:
        raise RuntimeError("Empty or oversized formula result")
    print(text.strip())


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(f"Local formula recognition failed: {error}", file=sys.stderr)
        sys.exit(1)
