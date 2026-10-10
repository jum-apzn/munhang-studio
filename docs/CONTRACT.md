# Shared contract (v1)
Project JSON: { version: 1, title: string, columns: 1|2, startNumber: integer, items: Item[] }
Item: { id: string, points: number, answerSpaceMm: number, blocks: Block[] }
Block tagged by `kind`: {kind:'text',text:string}, {kind:'equation',script:string}, {kind:'image',dataUrl:string,widthMm:number,alt:string}
Equation script is Hancom equation script (not LaTeX), editable in UI. UI may assist known basic conversions but never silently assert full conversion.
Rust serde uses camelCase. Core API `pub fn export_hwpx(project: &Project) -> Result<Vec<u8>, String>`.
Tauri commands: export_hwpx(project)->Vec<u8>, import_pdf(data:Vec<u8>)->Vec<String> (PNG data URLs), recognize_image(data:Vec<u8>)->String, runtime_status()->RuntimeStatus {pdf:bool,ocr:bool,korean:bool}.
Frontend uses window.__TAURI__.core.invoke when present. No server/network document upload. Browser preview supports image editing only and reports Rust PDF/OCR/export unavailable (never fake export).
Native save commands (preferred desktop UI): export_hwpx_to_file(project)->string|null and save_project_to_file(project)->string|null. Each opens OS save dialog, writes only selected path; null = cancel. Raw export_hwpx remains available for validation.
Optional formula support: runtime_status has formula:bool. recognize_formula(data)->{latex:string,script:string,warnings:string[]}. convert_formula(latex) returns the same struct without OCR. Unsupported LaTeX produces empty script plus warnings, preserving original latex. Desktop UI must not silently insert an empty/incorrect equation on warning.
Equation blocks optionally preserve sourceImage (PNG/JPEG data URL), sourceLatex (string), conversionWarnings (string array). These are editor-only provenance and are never emitted as duplicate pictures/text into HWPX. Draft JSON may contain an empty equation script; HWPX export rejects unfinished scripts.
Recovery: save_recovery_text_to_file(text)->string|null preserves raw failed-to-parse project text through the native save dialog, up to 80 MiB. Recovery mode must never silently replace unreadable stored data with a blank project.
