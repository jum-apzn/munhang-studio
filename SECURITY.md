# Security

Munhang Studio is an early local-first desktop application, not a security-hardened document sandbox. Use trusted documents and current PDF/OCR runtimes. Malformed PDFs and images are parsed by third-party native libraries. File size, page count, image dimensions, subprocess time, and selected output paths are bounded where practical; these limits do not eliminate parser vulnerabilities.

The renderer has no remote network permission in the production Content Security Policy. Rust custom commands accept bytes/project data, not user-supplied shell commands. Generated HWPX contains text, equation scripts and embedded PNG/JPEG assets; scripts/macros and remote image URLs are not supported.

Project JSON may include the user's original cropped document contents. Keep it private unless the user chooses to share it. Optional OCR model installation downloads public software/weights, never document images.

Do not publish private documents or credentials with issue reports. Report reproducible security concerns with a synthetic minimal example.
