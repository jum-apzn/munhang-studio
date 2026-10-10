# HWPX exporter implementation and verification

This is an original Rust serializer, not a copied document/template converter.

## Format references and licenses

- [Hancom format overview](https://tech.hancom.com/hwpxformat/): consulted for ZIP package parts and their roles. No text or illustrations copied.
- [Official Hancom OWPML model](https://github.com/hancom-io/hwpx-owpml-model): Apache-2.0, license inspected before reading implementation. Used to verify XML element/attribute names for EquationType, PictureType, AbstractShapeObjectType, AbstractShapeComponentType, ColumnDefType, SectionDefinitionType, ImageType, CharShapeType, ParaShapeType, StyleType, HWPMLHeadType and BorderFillType. No implementation files or templates incorporated.
- The proprietary HWP binary format is not implemented. No proprietary fonts or assets are bundled. Fonts are named in document styles; rendering depends on the receiving system's installed fonts.
- Third-party HWPX skills with noncommercial or unclear licensing were not incorporated.

## Output

- `mimetype` is the first ZIP member and is stored without compression.
- Package includes version, settings, container metadata, OPF manifest/spine, style header, section XML and preview text.
- Text remains text. Equation scripts become native `hp:equation` / `hp:script` objects; no equation screenshots.
- PNG/JPEG data URLs are decoded, content/dimensions checked, packaged in `BinData/`, and referenced by the OPF manifest and `hc:img`.
- A4, 15 mm margins, one or two newspaper-style columns. Long document text flows automatically. The title occupies the first column rather than spanning both columns.
- Number and points are editable text. Answer space is an empty paragraph with a fixed line-spacing style in HWP units (7200 units/inch).
- Image aspect ratio is preserved. Images shrink to column width and a 230 mm maximum height.
- No network fetches, macros, scripts, embedded fonts or remote resources.

## Tests and remaining acceptance gate

Unit tests cover ZIP integrity, XML parsing/escaping, image embedding/manifest/aspect ratio, equation script preservation, layout properties, JSON contract and invalid input rejection. The CLI creates `examples/sample-worksheet.hwpx` from original synthetic questions and a self-created diagram.

These are structural tests, **not a claim of complete HWPX schema or Hancom rendering conformance**. The independent Apache-2.0 `python-hwpx==6.8.0` validator was installed in an isolated test environment (not included as a runtime dependency). Its package checker, full-schema lint with known Hancom-2011 deviations suppressed, and editor-open safety/reopen check all passed with zero issues after fixes. See `examples/validation-report.json`. Its parser recovered all text paragraphs. This is independent parsing, not Hancom application execution. No Hancom runtime is available in this environment. Equation width/height/baseline are initial estimates; multi-line equations and unusually large scripts may need adjustment in Hancom. Automatic pagination, font fallback, picture visibility, exact answer spacing and saving/reopening editable equations need manual verification in a current Hancom Office installation before production use.

Acceptance steps: open the sample, check title and both columns, inspect the graph, double-click and modify each equation, change text and points, save as HWPX, close, reopen, and print/export PDF to check page breaks. Do not describe round-trip verification as complete until this has been performed.


## Independent validation reproduction

In a separate Python virtual environment, install `python-hwpx==6.8.0` from PyPI and run:

    python scripts/validate_hwpx.py examples/sample-worksheet.hwpx

The library is Apache-2.0 and was used only as a test tool; no library source or templates are bundled. The report also records raw 2024-schema deviations before the validator suppresses its known Hancom-2011 compatibility differences: shadeColor `none`, autoSpacing/docOption placement, legacy tabStop, and picture transformation/point/margin elements. Therefore zero filtered lint issues must not be presented as zero raw 2024-XSD deviations or full standards conformance.
