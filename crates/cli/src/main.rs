use munhang_core::{export_hwpx, Project};
use std::{env, fs, path::Path, process};
fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("export") if args.len()==4 => {
            let data = fs::read(&args[2]).map_err(|e|e.to_string())?;
            if data.len()>64*1024*1024 { return Err("Project exceeds 64 MiB".into()); }
            let project: Project=serde_json::from_slice(&data).map_err(|e|e.to_string())?;
            let output=export_hwpx(&project)?;
            if Path::new(&args[3]).exists() { return Err("Output exists. Choose a new path to avoid overwriting.".into()); }
            fs::write(&args[3],output).map_err(|e|e.to_string())?;
            println!("Created {}",args[3]);
            Ok(())
        },
        Some("import-pdf") if args.len()==4 => {
            if Path::new(&args[3]).exists() { return Err("Output exists. Choose a new path.".into()); }
            let data=fs::read(&args[2]).map_err(|e|e.to_string())?;
            let pages=munhang_core::runtime::import_pdf(&data)?;
            let bytes=serde_json::to_vec(&pages).map_err(|e|e.to_string())?;
            fs::write(&args[3],bytes).map_err(|e|e.to_string())?;
            println!("Imported {} pages",pages.len()); Ok(())
        },
        Some("formula") if args.len()==3 => {
            let data=fs::read(&args[2]).map_err(|e|e.to_string())?;
            let latex=munhang_core::runtime::recognize_formula(&data)?;
            let converted=munhang_core::formula::convert_latex(&latex);
            println!("{}",serde_json::to_string_pretty(&converted).map_err(|e|e.to_string())?); Ok(())
        },
        Some("convert-formula") if args.len()==3 => {
            println!("{}",serde_json::to_string_pretty(&munhang_core::formula::convert_latex(&args[2])).map_err(|e|e.to_string())?); Ok(())
        },
        Some("status") => { println!("{}",serde_json::to_string_pretty(&munhang_core::runtime::runtime_status()).map_err(|e|e.to_string())?); Ok(()) },
        Some("ocr") if args.len()==3 => {let data=fs::read(&args[2]).map_err(|e|e.to_string())?; println!("{}",munhang_core::runtime::recognize_image(&data)?); Ok(())},
        _=>Err("Usage: munhang-cli export project.json output.hwpx | status | ocr image.png | import-pdf input.pdf pages.json | formula image.png | convert-formula 'LaTeX'".into())
    }
}
fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        process::exit(1);
    }
}
