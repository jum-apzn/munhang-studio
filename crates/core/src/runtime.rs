//! Offline system-tool adapters. No network requests or silently downloaded models.
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Serialize;
use std::{
    fs,
    io::{Cursor, Read},
    path::Path,
    process::{Command, Stdio},
    sync::Mutex,
    thread,
    time::{Duration, Instant},
};

const MAX_INPUT: usize = 40 * 1024 * 1024;
const MAX_PAGES: usize = 20;
const MAX_PIXELS: u64 = 25_000_000;
const MAX_OUTPUT: u64 = 180 * 1024 * 1024;
static WORKER: Mutex<()> = Mutex::new(());

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStatus {
    pub pdf: bool,
    pub ocr: bool,
    pub korean: bool,
    pub formula: bool,
}

// Drain both pipes concurrently so a verbose child cannot block waiting for a pipe.
// Keep only a bounded prefix in RAM; continue draining until the child exits.
fn drain(mut pipe: impl Read) -> Vec<u8> {
    let mut kept = Vec::new();
    let mut buffer = [0u8; 8192];
    while let Ok(n) = pipe.read(&mut buffer) {
        if n == 0 {
            break;
        }
        let remaining = (1024 * 1024 + 1usize).saturating_sub(kept.len());
        kept.extend_from_slice(&buffer[..n.min(remaining)]);
    }
    kept
}

fn directory_size(path: &Path) -> u64 {
    fs::read_dir(path)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

fn run(mut command: Command, timeout: Duration, scratch: Option<&Path>) -> Result<Vec<u8>, String> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("OMP_THREAD_LIMIT", "2");
    let mut child = command
        .spawn()
        .map_err(|e| format!("로컬 실행 도구를 시작하지 못했습니다: {e}"))?;
    let out = child.stdout.take().ok_or("표준 출력 연결 실패")?;
    let err = child.stderr.take().ok_or("오류 출력 연결 실패")?;
    let out_thread = thread::spawn(move || drain(out));
    let err_thread = thread::spawn(move || drain(err));
    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Err(e) => break Err(format!("로컬 도구 상태 확인 실패: {e}")),
            Ok(None) => {}
        }
        if started.elapsed() > timeout {
            break Err(
                "처리 제한 시간을 초과했습니다. 페이지/영역을 줄여 다시 시도하세요.".to_owned(),
            );
        }
        if scratch.is_some_and(|p| directory_size(p) > MAX_OUTPUT) {
            break Err("변환 결과가 용량 제한을 초과했습니다.".to_owned());
        }
        thread::sleep(Duration::from_millis(40));
    };
    if status.is_err() {
        let _ = child.kill();
        let _ = child.wait();
    }
    let stdout = out_thread.join().unwrap_or_default();
    let stderr = err_thread.join().unwrap_or_default();
    let status = status?;
    if !status.success() {
        let detail = String::from_utf8_lossy(&stderr[..stderr.len().min(2000)]);
        return Err(format!("로컬 도구가 실패했습니다: {detail}"));
    }
    if stdout.len() > 1024 * 1024 {
        return Err("인식 텍스트가 1 MB 제한을 초과했습니다.".to_owned());
    }
    Ok(stdout)
}

fn languages() -> Option<Vec<String>> {
    let mut cmd = Command::new("tesseract");
    cmd.arg("--list-langs");
    run(cmd, Duration::from_secs(5), None).ok().map(|s| {
        String::from_utf8_lossy(&s)
            .lines()
            .map(str::trim)
            .map(str::to_owned)
            .collect()
    })
}

pub fn runtime_status() -> RuntimeStatus {
    let mut pdf = Command::new("pdftoppm");
    pdf.arg("-v");
    let langs = languages().unwrap_or_default();
    RuntimeStatus {
        pdf: run(pdf, Duration::from_secs(5), None).is_ok(),
        ocr: langs.iter().any(|l| l == "eng" || l == "kor"),
        korean: langs.iter().any(|l| l == "kor"),
        formula: formula_config().is_ok(),
    }
}

fn validate_size(data: &[u8]) -> Result<(), String> {
    if data.is_empty() {
        return Err("빈 파일입니다.".to_owned());
    }
    if data.len() > MAX_INPUT {
        return Err("파일은 40 MB 이하여야 합니다.".to_owned());
    }
    Ok(())
}

/// Rasterize all pages, rejecting (rather than silently truncating) PDFs over 20 pages.
/// Each page is capped to 2200 px on its longest edge. Original PDF stays unmodified.
pub fn import_pdf(data: &[u8]) -> Result<Vec<String>, String> {
    validate_size(data)?;
    if !data.starts_with(b"%PDF-") {
        return Err("올바른 PDF 파일이 아닙니다.".to_owned());
    }
    let _guard = WORKER
        .try_lock()
        .map_err(|_| "다른 PDF/OCR 작업이 진행 중입니다. 잠시 후 다시 시도하세요.")?;
    let dir = tempfile::Builder::new()
        .prefix("munhang-pdf-")
        .tempdir()
        .map_err(|e| e.to_string())?;
    let input = dir.path().join("input.pdf");
    let prefix = dir.path().join("page");
    fs::write(&input, data).map_err(|e| e.to_string())?;
    let mut cmd = Command::new("pdftoppm");
    cmd.current_dir(dir.path())
        .args([
            "-png",
            "-r",
            "160",
            "-scale-to",
            "2200",
            "-f",
            "1",
            "-l",
            "21",
        ])
        .arg(&input)
        .arg(&prefix);
    run(cmd, Duration::from_secs(90), Some(dir.path()))?;
    if directory_size(dir.path()) > MAX_OUTPUT {
        return Err("변환 결과가 용량 제한을 초과했습니다.".to_owned());
    }
    let mut pages: Vec<_> = fs::read_dir(dir.path())
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .filter_map(|e| {
            let path = e.path();
            let name = path.file_name()?.to_str()?;
            let number = name
                .strip_prefix("page-")?
                .strip_suffix(".png")?
                .parse::<usize>()
                .ok()?;
            Some((number, path))
        })
        .collect();
    pages.sort_by_key(|(number, _)| *number);
    if pages.len() > MAX_PAGES {
        return Err("PDF는 한 번에 20쪽까지 가져올 수 있습니다. PDF를 나누어 주세요.".to_owned());
    }
    if pages.is_empty() {
        return Err("PDF에서 페이지를 읽지 못했습니다.".to_owned());
    }
    pages
        .into_iter()
        .map(|(_, path)| {
            let png = fs::read(path).map_err(|e| e.to_string())?;
            if !png.starts_with(b"\x89PNG\r\n\x1a\n") {
                return Err("PDF 변환 도구가 올바른 PNG를 생성하지 못했습니다.".to_owned());
            }
            Ok(format!("data:image/png;base64,{}", STANDARD.encode(png)))
        })
        .collect()
}

/// Plain-text OCR only. This does not recognize equations as editable Hancom objects.
/// Missing Korean data must be indicated by runtime_status() in the UI.
pub fn recognize_image(data: &[u8]) -> Result<String, String> {
    validate_size(data)?;
    let _guard = WORKER
        .try_lock()
        .map_err(|_| "다른 PDF/OCR 작업이 진행 중입니다. 잠시 후 다시 시도하세요.")?;
    let dimensions = image::ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .map_err(|e| e.to_string())?
        .into_dimensions()
        .map_err(|e| format!("이미지 형식 확인 실패: {e}"))?;
    if u64::from(dimensions.0) * u64::from(dimensions.1) > MAX_PIXELS
        || dimensions.0 == 0
        || dimensions.1 == 0
    {
        return Err(
            "이미지는 2,500만 화소 이하여야 합니다. 영역을 잘라 다시 시도하세요.".to_owned(),
        );
    }
    let mut reader = image::ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .map_err(|e| e.to_string())?;
    let mut limits = image::Limits::default();
    limits.max_alloc = Some(150 * 1024 * 1024);
    reader.limits(limits);
    let img = reader
        .decode()
        .map_err(|e| format!("이미지 해독 실패: {e}"))?;
    let langs =
        languages().ok_or("Tesseract가 설치되어 있지 않습니다. docs/OCR.md를 확인하세요.")?;
    let korean = langs.iter().any(|l| l == "kor");
    let english = langs.iter().any(|l| l == "eng");
    let language = match (korean, english) {
        (true, true) => "kor+eng",
        (true, false) => "kor",
        (false, true) => "eng",
        _ => return Err("Tesseract의 kor 또는 eng 언어 데이터가 필요합니다.".to_owned()),
    };
    let dir = tempfile::Builder::new()
        .prefix("munhang-ocr-")
        .tempdir()
        .map_err(|e| e.to_string())?;
    let input = dir.path().join("input.png");
    img.save_with_format(&input, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    let mut cmd = Command::new("tesseract");
    cmd.current_dir(dir.path())
        .arg(&input)
        .args(["stdout", "-l", language, "--psm", "6"]);
    let output = run(cmd, Duration::from_secs(60), Some(dir.path()))?;
    let text = String::from_utf8(output).map_err(|_| "OCR 결과가 올바른 UTF-8이 아닙니다.")?;
    if text.trim().is_empty() {
        return Err(
            "읽을 수 있는 텍스트가 없습니다. 더 선명한 영역을 선택하거나 직접 입력하세요."
                .to_owned(),
        );
    }
    Ok(text.trim().to_owned())
}

fn formula_config() -> Result<(std::path::PathBuf, std::path::PathBuf), String> {
    let python = std::env::var_os("MUNHANG_FORMULA_PYTHON")
        .map(std::path::PathBuf::from)
        .ok_or("수식 OCR용 Python 경로가 설정되지 않았습니다. docs/OCR.md를 확인하세요.")?;
    let model = std::env::var_os("MUNHANG_FORMULA_MODEL_DIR")
        .map(std::path::PathBuf::from)
        .ok_or("수식 OCR 모델 폴더가 설정되지 않았습니다.")?;
    if !python.is_absolute()
        || !python.is_file()
        || !model.is_absolute()
        || ![
            "config.json",
            "inference.json",
            "inference.pdiparams",
            "inference.yml",
        ]
        .iter()
        .all(|f| model.join(f).is_file())
    {
        return Err("수식 OCR 설정에 실행 파일 또는 공식 모델 파일이 누락되었습니다.".to_owned());
    }
    Ok((python, model))
}

/// Optional PP-FormulaNet_plus-S local CPU sidecar. Returns raw LaTeX only.
/// The caller must preserve the original and explicitly review Hancom conversion.
pub fn recognize_formula(data: &[u8]) -> Result<String, String> {
    validate_size(data)?;
    let _guard = WORKER
        .try_lock()
        .map_err(|_| "다른 PDF/OCR 작업이 진행 중입니다. 잠시 후 다시 시도하세요.")?;
    let (python, model) = formula_config()?;
    let dims = image::ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .map_err(|e| e.to_string())?
        .into_dimensions()
        .map_err(|e| format!("이미지 형식 확인 실패: {e}"))?;
    if u64::from(dims.0) * u64::from(dims.1) > MAX_PIXELS || dims.0 == 0 || dims.1 == 0 {
        return Err("수식 영역은 2,500만 화소 이하여야 합니다.".to_owned());
    }
    let mut reader = image::ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .map_err(|e| e.to_string())?;
    let mut limits = image::Limits::default();
    limits.max_alloc = Some(150 * 1024 * 1024);
    reader.limits(limits);
    let img = reader.decode().map_err(|e| e.to_string())?;
    let dir = tempfile::Builder::new()
        .prefix("munhang-formula-")
        .tempdir()
        .map_err(|e| e.to_string())?;
    let input = dir.path().join("input.png");
    let sidecar = dir.path().join("formula.py");
    img.save_with_format(&input, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    fs::write(&sidecar, include_str!("../../../scripts/setup-ocr.py"))
        .map_err(|e| e.to_string())?;
    let mut cmd = Command::new(python);
    // PaddleX 3.7.2 utils/cache.py explicitly honors this cache root.
    cmd.env("PADDLE_PDX_CACHE_HOME", dir.path().join("paddlex-cache"))
        .env("PADDLE_EXTENSION_DIR", dir.path().join("paddle-extensions"));
    cmd.current_dir(dir.path())
        .arg(&sidecar)
        .arg("--model-dir")
        .arg(model)
        .arg("--input")
        .arg(&input);
    let output = run(cmd, Duration::from_secs(120), Some(dir.path()))?;
    let latex = String::from_utf8(output).map_err(|_| "수식 OCR 결과가 UTF-8이 아닙니다.")?;
    if latex.trim().is_empty() || latex.len() > 100_000 {
        return Err("수식 인식 결과가 비었거나 너무 깁니다.".to_owned());
    }
    Ok(latex.trim().to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_bad_inputs() {
        assert!(import_pdf(b"not a pdf").is_err());
        assert!(recognize_image(b"not an image").is_err());
        assert!(validate_size(&[]).is_err());
    }
    #[test]
    fn serializes_status_contract() {
        let status = RuntimeStatus {
            pdf: false,
            ocr: true,
            korean: false,
            formula: false,
        };
        let json = serde_json::to_value(status).unwrap();
        assert_eq!(json["korean"], false);
        assert_eq!(json.as_object().unwrap().len(), 4);
    }
    #[test]
    fn bounds_input_size() {
        assert_eq!(MAX_PAGES, 20);
        assert!(validate_size(&vec![0; MAX_INPUT + 1]).is_err());
    }
    #[test]
    #[ignore = "Requires local engines and MUNHANG_FIXTURES with Korean/English PNG and PDFs"]
    fn local_engine_smoke_test() {
        let base =
            std::path::PathBuf::from(std::env::var("MUNHANG_FIXTURES").expect("fixture directory"));
        let status = runtime_status();
        assert!(status.pdf && status.ocr && status.korean);
        let korean = recognize_image(&fs::read(base.join("korean.png")).unwrap()).unwrap();
        assert!(
            korean.contains("수학 문제를 풀고 정답을 쓰시오."),
            "{korean}"
        );
        let english = recognize_image(&fs::read(base.join("english.png")).unwrap()).unwrap();
        assert!(english.contains("Mathematics worksheet 123"), "{english}");
        let pages = import_pdf(&fs::read(base.join("one-page.pdf")).unwrap()).unwrap();
        assert_eq!(pages.len(), 1);
        assert!(pages[0].starts_with("data:image/png;base64,iVBOR"));
        let long = import_pdf(&fs::read(base.join("too-many.pdf")).unwrap()).unwrap_err();
        assert!(long.contains("20쪽"), "{long}");
    }

    #[test]
    #[ignore = "Requires configured optional Paddle CPU sidecar and formula.png fixture"]
    fn local_formula_smoke_test() {
        let base =
            std::path::PathBuf::from(std::env::var("MUNHANG_FIXTURES").expect("fixture directory"));
        assert!(runtime_status().formula);
        let latex = recognize_formula(&fs::read(base.join("formula.png")).unwrap()).unwrap();
        assert_eq!(latex.replace(' ', ""), "x^{2}+2x+1=(x+1)^{2}");
    }
    #[test]
    #[cfg(unix)]
    fn subprocess_timeout_kills_child() {
        let mut cmd = Command::new("sleep");
        cmd.arg("5");
        let start = Instant::now();
        let err = run(cmd, Duration::from_millis(80), None).unwrap_err();
        assert!(err.contains("제한 시간"), "{err}");
        assert!(start.elapsed() < Duration::from_secs(3));
    }
}
