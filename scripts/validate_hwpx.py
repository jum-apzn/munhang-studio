#!/usr/bin/env python3
"""Optional independent checker. Requires python-hwpx==6.8.0 (Apache-2.0).
Install in a separate test venv; not an application runtime dependency.
This checker does NOT launch Hancom Office or certify rendering.
"""
import argparse
import dataclasses
import json
import sys
from pathlib import Path

from hwpx import HwpxDocument, validate_editor_open_safety, validate_package
from hwpx.tools.validator import validate_document

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("source", type=Path)
args = parser.parse_args()
package = validate_package(args.source)
schema = validate_document(args.source)
safety = validate_editor_open_safety(args.source)
doc = HwpxDocument.open(args.source)
report = {
    "validator": "python-hwpx 6.8.0",
    "package": dataclasses.asdict(package),
    "schema": dataclasses.asdict(schema),
    "editor_open_safety": dataclasses.asdict(safety),
    "parsed_sections": len(doc.sections),
    "note": "Schema lint tolerates known 2011 Hancom differences from OWPML 2024. Hancom rendering/edit round-trip remains unverified.",
}
print(json.dumps(report, ensure_ascii=False, indent=2))
sys.exit(1 if package.issues or schema.issues or not safety.reopen_ok else 0)
