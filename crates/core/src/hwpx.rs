//! Original OWPML/HWPX writer. Format references (no source/template copied):
//! https://tech.hancom.com/hwpxformat/
//! https://github.com/hancom-io/hwpx-owpml-model (Apache-2.0)
//! Hancom rendering and equation metrics must still be checked in Hancom Office.
use crate::{Block, Project};
use base64::{engine::general_purpose::STANDARD, Engine};
use std::io::{Cursor, Write};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

const XML: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>";
const NS: &str = r#"xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph" xmlns:hc="http://www.hancom.co.kr/hwpml/2011/core" xmlns:hh="http://www.hancom.co.kr/hwpml/2011/head" xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section" xmlns:ha="http://www.hancom.co.kr/hwpml/2011/app" xmlns:hp10="http://www.hancom.co.kr/hwpml/2016/paragraph" xmlns:hhs="http://www.hancom.co.kr/hwpml/2011/history" xmlns:hm="http://www.hancom.co.kr/hwpml/2011/master-page" xmlns:hpf="http://www.hancom.co.kr/schema/2011/hpf" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:opf="http://www.idpf.org/2007/opf/" xmlns:ooxmlchart="http://www.hancom.co.kr/hwpml/2016/ooxmlchart" xmlns:hwpunitchar="http://www.hancom.co.kr/hwpml/2016/HwpUnitChar" xmlns:epub="http://www.idpf.org/2007/ops" xmlns:config="urn:oasis:names:tc:opendocument:xmlns:config:1.0""#;
const MAX_ASSET: usize = 32 * 1024 * 1024;
const MAX_TOTAL: usize = 128 * 1024 * 1024;

/// Export an unencrypted, macro-free HWPX archive entirely in memory.
/// Text and equation scripts are native editable objects; pictures remain pictures.
pub fn export_hwpx(project: &Project) -> Result<Vec<u8>, String> {
    validate(project)?;
    let mut assets = Vec::new();
    let section = section(project, &mut assets)?;
    let header = header(project);
    let mut manifest = format!(
        r#"{XML}<opf:package xmlns:opf="http://www.idpf.org/2007/opf/" version="3.0" unique-identifier="document_id"><opf:metadata><opf:title>{}</opf:title><opf:language>ko</opf:language><opf:meta name="creator" content="text">Munhang Studio</opf:meta></opf:metadata><opf:manifest><opf:item id="version" href="version.xml" media-type="application/xml"/><opf:item id="header" href="Contents/header.xml" media-type="application/xml"/><opf:item id="section0" href="Contents/section0.xml" media-type="application/xml"/><opf:item id="settings" href="settings.xml" media-type="application/xml"/>"#,
        esc(&project.title)
    );
    for a in &assets {
        manifest.push_str(&format!(
            r#"<opf:item id="{}" href="{}" media-type="{}" isEmbeded="1"/>"#,
            a.id, a.path, a.mime
        ));
    }
    manifest.push_str(r#"</opf:manifest><opf:spine><opf:itemref idref="header" linear="yes"/><opf:itemref idref="section0" linear="yes"/></opf:spine></opf:package>"#);
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    put(
        &mut zip,
        "mimetype",
        b"application/hwp+zip",
        CompressionMethod::Stored,
    )?;
    put(&mut zip, "version.xml", format!(r#"{XML}<hv:HCFVersion xmlns:hv="http://www.hancom.co.kr/hwpml/2011/version" tagetApplication="WORDPROCESSOR" major="5" minor="1" micro="0" buildNumber="1" os="1" xmlVersion="1.4" application="Munhang Studio" appVersion="0.1.0"/>"#).as_bytes(), CompressionMethod::Deflated)?;
    put(&mut zip, "META-INF/container.xml", format!(r#"{XML}<container xmlns="urn:oasis:names:tc:opendocument:xmlns:container" version="1.0"><rootfiles><rootfile full-path="Contents/content.hpf" media-type="application/hwpml-package+xml"/><rootfile full-path="Preview/PrvText.txt" media-type="text/plain"/></rootfiles></container>"#).as_bytes(), CompressionMethod::Deflated)?;
    put(&mut zip, "META-INF/manifest.xml", format!(r#"{XML}<manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0"/>"#).as_bytes(), CompressionMethod::Deflated)?;
    put(
        &mut zip,
        "META-INF/container.rdf",
        format!(r#"{XML}<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"/>"#)
            .as_bytes(),
        CompressionMethod::Deflated,
    )?;
    put(&mut zip, "settings.xml", format!(r#"{XML}<ha:HWPApplicationSetting xmlns:ha="http://www.hancom.co.kr/hwpml/2011/app"><ha:CaretPosition listIDRef="0" paraIDRef="0" pos="0"/></ha:HWPApplicationSetting>"#).as_bytes(), CompressionMethod::Deflated)?;
    for (path, data) in [
        ("Contents/content.hpf", manifest),
        ("Contents/header.xml", header),
        ("Contents/section0.xml", section),
        ("Preview/PrvText.txt", preview(project)),
    ] {
        put(&mut zip, path, data.as_bytes(), CompressionMethod::Deflated)?;
    }
    for asset in assets {
        put(
            &mut zip,
            &asset.path,
            &asset.bytes,
            CompressionMethod::Deflated,
        )?;
    }
    zip.finish()
        .map(|x| x.into_inner())
        .map_err(|e| format!("HWPX ZIP finalization failed: {e}"))
}

fn put(
    zip: &mut ZipWriter<Cursor<Vec<u8>>>,
    path: &str,
    data: &[u8],
    compression: CompressionMethod,
) -> Result<(), String> {
    zip.start_file(
        path,
        SimpleFileOptions::default().compression_method(compression),
    )
    .map_err(|e| e.to_string())?;
    zip.write_all(data).map_err(|e| e.to_string())
}
fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}
fn valid_text(s: &str) -> bool {
    s.chars().all(|c| {
        matches!(c, '\t' | '\n' | '\r') || (c >= '\u{20}' && c != '\u{fffe}' && c != '\u{ffff}')
    })
}
fn validate(p: &Project) -> Result<(), String> {
    if p.version != 1 {
        return Err("Unsupported project version; expected 1".into());
    }
    if !matches!(p.columns, 1 | 2) {
        return Err("Columns must be 1 or 2".into());
    }
    if p.start_number == 0
        || p.items.len() > 1000
        || p.start_number.checked_add(p.items.len() as u32).is_none()
    {
        return Err("Invalid numbering or more than 1000 items".into());
    }
    if p.title.len() > 4096 || !valid_text(&p.title) {
        return Err("Title is too long or contains invalid XML characters".into());
    }
    let mut text_bytes = p.title.len();
    for item in &p.items {
        if !item.points.is_finite() || !(0.0..=10000.0).contains(&item.points) {
            return Err("Points must be between 0 and 10000".into());
        }
        if !item.answer_space_mm.is_finite() || !(0.0..=230.0).contains(&item.answer_space_mm) {
            return Err("Answer space must be between 0 and 230 mm".into());
        }
        if item.blocks.len() > 1000 {
            return Err("Too many blocks in an item".into());
        }
        for b in &item.blocks {
            let s = match b {
                Block::Text { text } => text,
                Block::Equation { script, .. } => {
                    if script.trim().is_empty() || script.len() > 16000 {
                        return Err("Equation script is empty or too long".into());
                    }
                    script
                }
                Block::Image {
                    data_url,
                    width_mm,
                    alt,
                } => {
                    if !width_mm.is_finite() || !(1.0..=180.0).contains(width_mm) {
                        return Err("Image width must be between 1 and 180 mm".into());
                    }
                    if data_url.len() > MAX_ASSET * 4 / 3 + 128 {
                        return Err("Image exceeds 32 MiB limit".into());
                    }
                    alt
                }
            };
            if !valid_text(s) {
                return Err("Content contains characters forbidden by XML 1.0".into());
            }
            text_bytes += s.len();
            if text_bytes > 8 * 1024 * 1024 {
                return Err("Document text exceeds 8 MiB limit".into());
            }
        }
    }
    Ok(())
}
fn mm(value: f64) -> u32 {
    (value * 7200.0 / 25.4).round() as u32
}
fn para(id: u32, style: u32, run: &str) -> String {
    format!(
        r#"<hp:p id="{id}" paraPrIDRef="{style}" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0"><hp:run charPrIDRef="0">{run}</hp:run></hp:p>"#
    )
}
fn text_run(text: &str) -> String {
    format!(
        "<hp:t>{}</hp:t>",
        text.split('\n')
            .map(esc)
            .collect::<Vec<_>>()
            .join("<hp:lineBreak/>")
    )
}
fn section(p: &Project, assets: &mut Vec<Asset>) -> Result<String, String> {
    let mut xml = format!("{XML}<hs:sec {NS}>");
    let props = format!(
        r#"<hp:secPr id="0" textDirection="HORIZONTAL" spaceColumns="0" tabStop="8000" tabStopVal="4000" tabStopUnit="HWPUNIT" outlineShapeIDRef="0" memoShapeIDRef="0" textVerticalWidthHead="0" masterPageCnt="0"><hp:grid lineGrid="0" charGrid="0" wonggojiFormat="0"/><hp:startNum pageStartsOn="BOTH" page="0" pic="0" tbl="0" equation="0"/><hp:visibility hideFirstHeader="0" hideFirstFooter="0" hideFirstMasterPage="0" border="SHOW_ALL" fill="SHOW_ALL" hideFirstPageNum="0" hideFirstEmptyLine="0" showLineNumber="0"/><hp:pagePr landscape="WIDELY" width="59528" height="84188" gutterType="LEFT_ONLY"><hp:margin header="2835" footer="2835" gutter="0" left="4252" right="4252" top="4252" bottom="4252"/></hp:pagePr></hp:secPr><hp:ctrl><hp:colPr id="0" type="NEWSPAPER" layout="LEFT" colCount="{}" sameSz="1" sameGap="2268"/></hp:ctrl>"#,
        p.columns
    );
    xml.push_str(&para(0, 1, &(props + &text_run(&p.title))));
    let mut id = 1;
    let mut object_id = 100000;
    for (n, item) in p.items.iter().enumerate() {
        xml.push_str(&para(
            id,
            2,
            &text_run(&format!(
                "{}.  [{}점]",
                p.start_number + n as u32,
                item.points
            )),
        ));
        id += 1;
        for b in &item.blocks {
            let run = match b {
                Block::Text { text } => text_run(text),
                Block::Equation { script, .. } => {
                    object_id += 1;
                    // Initial metrics only. Hancom recomputes equation layout on editing.
                    let width = ((script.chars().count() as u32 * 550).max(2400))
                        .min(if p.columns == 2 { 24000 } else { 50000 });
                    format!(
                        r##"<hp:equation id="{object_id}" zOrder="0" numberingType="EQUATION" textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" lock="0" dropcapstyle="None" version="Equation Version 60" baseLine="85" textColor="#000000" baseUnit="1100" lineMode="CHAR" font="HYhwpEQ">{}<hp:script>{}</hp:script></hp:equation><hp:t/>"##,
                        shape_position(width, 2200),
                        esc(script)
                    )
                }
                Block::Image {
                    data_url,
                    width_mm,
                    alt,
                } => {
                    let asset = decode_asset(data_url, assets.len() + 1)?;
                    if assets.iter().map(|a| a.bytes.len()).sum::<usize>() + asset.bytes.len()
                        > MAX_TOTAL
                    {
                        return Err("Total images exceed 128 MiB limit".into());
                    }
                    let column_mm = if p.columns == 2 { 86.0 } else { 180.0 };
                    let mut width = mm(width_mm.min(column_mm));
                    let mut height =
                        (width as f64 * asset.height as f64 / asset.width as f64).round() as u32;
                    if height > mm(230.0) {
                        width = (width as f64 * mm(230.0) as f64 / height as f64).round() as u32;
                        height = mm(230.0);
                    }
                    object_id += 1;
                    let run = picture(object_id, &asset.id, width, height, alt);
                    assets.push(asset);
                    run
                }
            };
            xml.push_str(&para(id, 0, &run));
            id += 1;
        }
        if item.answer_space_mm > 0.0 {
            xml.push_str(&para(id, 3 + n as u32, "<hp:t/>"));
            id += 1;
        }
    }
    xml.push_str("</hs:sec>");
    Ok(xml)
}
fn shape_position(w: u32, h: u32) -> String {
    format!(
        r#"<hp:sz width="{w}" widthRelTo="ABSOLUTE" height="{h}" heightRelTo="ABSOLUTE" protect="0"/><hp:pos treatAsChar="1" affectLSpacing="1" flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" vertRelTo="PARA" horzRelTo="PARA" vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/><hp:outMargin left="0" right="0" top="0" bottom="0"/>"#
    )
}
struct Asset {
    id: String,
    path: String,
    mime: &'static str,
    bytes: Vec<u8>,
    width: u32,
    height: u32,
}
fn decode_asset(url: &str, index: usize) -> Result<Asset, String> {
    let (prefix, payload) = url
        .split_once(',')
        .ok_or("Image must be a base64 PNG/JPEG data URL")?;
    let (ext, mime, format) = match prefix {
        "data:image/png;base64" => ("png", "image/png", image::ImageFormat::Png),
        "data:image/jpeg;base64" => ("jpg", "image/jpeg", image::ImageFormat::Jpeg),
        _ => return Err("Only embedded base64 PNG and JPEG images are supported".into()),
    };
    let bytes = STANDARD
        .decode(payload)
        .map_err(|_| "Invalid image base64")?;
    if bytes.len() > MAX_ASSET {
        return Err("Image exceeds 32 MiB limit".into());
    }
    if image::guess_format(&bytes).map_err(|_| "Unrecognized image")? != format {
        return Err("Image content does not match its declared type".into());
    }
    let (width, height) = image::ImageReader::with_format(Cursor::new(&bytes), format)
        .into_dimensions()
        .map_err(|_| "Invalid image dimensions")?;
    if width == 0
        || height == 0
        || width > 20000
        || height > 20000
        || width as u64 * height as u64 > 100_000_000
    {
        return Err("Image dimensions exceed safety limits".into());
    }
    // Decode once with library limits: prevents exporting broken image payloads.
    image::ImageReader::with_format(Cursor::new(&bytes), format)
        .decode()
        .map_err(|e| format!("Invalid image: {e}"))?;
    Ok(Asset {
        id: format!("image{index}"),
        path: format!("BinData/image{index}.{ext}"),
        mime,
        bytes,
        width,
        height,
    })
}
fn picture(id: u32, asset: &str, w: u32, h: u32, alt: &str) -> String {
    format!(
        r#"<hp:pic id="{id}" zOrder="0" numberingType="PICTURE" textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" lock="0" dropcapstyle="None" href="" groupLevel="0" instid="{id}" reverse="0"><hp:offset x="0" y="0"/><hp:orgSz width="{w}" height="{h}"/><hp:curSz width="{w}" height="{h}"/><hp:flip horizontal="0" vertical="0"/><hp:rotationInfo angle="0" centerX="{}" centerY="{}" rotateimage="1"/><hp:renderingInfo><hc:transMatrix e1="1" e2="0" e3="0" e4="0" e5="1" e6="0"/><hc:scaMatrix e1="1" e2="0" e3="0" e4="0" e5="1" e6="0"/><hc:rotMatrix e1="1" e2="0" e3="0" e4="0" e5="1" e6="0"/></hp:renderingInfo><hp:imgRect><hc:pt0 x="0" y="0"/><hc:pt1 x="{w}" y="0"/><hc:pt2 x="{w}" y="{h}"/><hc:pt3 x="0" y="{h}"/></hp:imgRect><hp:imgClip left="0" right="{w}" top="0" bottom="{h}"/><hp:inMargin left="0" right="0" top="0" bottom="0"/><hp:imgDim dimwidth="{w}" dimheight="{h}"/><hc:img binaryItemIDRef="{asset}" bright="0" contrast="0" effect="REAL_PIC" alpha="0"/>{}<hp:shapeComment>{}</hp:shapeComment></hp:pic><hp:t/>"#,
        w / 2,
        h / 2,
        shape_position(w, h),
        esc(alt)
    )
}
fn preview(p: &Project) -> String {
    let mut s = format!("{}\n", p.title);
    for (n, item) in p.items.iter().enumerate() {
        s.push_str(&format!(
            "\n{}. [{}점]\n",
            p.start_number + n as u32,
            item.points
        ));
        for b in &item.blocks {
            s.push_str(match b {
                Block::Text { text } => text,
                Block::Equation { script, .. } => script,
                Block::Image { alt, .. } => alt,
            });
            s.push('\n');
        }
    }
    s
}
fn header(p: &Project) -> String {
    let mut s = format!(
        r#"{XML}<hh:head {NS} version="1.4" secCnt="1"><hh:beginNum page="1" footnote="1" endnote="1" pic="1" tbl="1" equation="1"/><hh:refList><hh:fontfaces itemCnt="7">"#
    );
    for lang in [
        "HANGUL", "LATIN", "HANJA", "JAPANESE", "OTHER", "SYMBOL", "USER",
    ] {
        s.push_str(&format!(r#"<hh:fontface lang="{lang}" fontCnt="1"><hh:font id="0" face="함초롬바탕" type="TTF" isEmbedded="0"><hh:typeInfo familyType="FCAT_MYUNGJO" weight="6" proportion="0" contrast="0" strokeVariation="1" armStyle="1" letterform="1" midline="1" xHeight="1"/></hh:font></hh:fontface>"#));
    }
    s.push_str(r##"</hh:fontfaces><hh:borderFills itemCnt="1"><hh:borderFill id="1" threeD="0" shadow="0" centerLine="NONE" breakCellSeparateLine="0"><hh:slash type="NONE" Crooked="0" isCounter="0"/><hh:backSlash type="NONE" Crooked="0" isCounter="0"/><hh:leftBorder type="NONE" width="0.1 mm" color="#000000"/><hh:rightBorder type="NONE" width="0.1 mm" color="#000000"/><hh:topBorder type="NONE" width="0.1 mm" color="#000000"/><hh:bottomBorder type="NONE" width="0.1 mm" color="#000000"/><hh:diagonal type="SOLID" width="0.1 mm" color="#000000"/></hh:borderFill></hh:borderFills><hh:charProperties itemCnt="1"><hh:charPr id="0" height="1100" textColor="#000000" shadeColor="none" useFontSpace="0" useKerning="0" symMark="NONE" borderFillIDRef="1"><hh:fontRef hangul="0" latin="0" hanja="0" japanese="0" other="0" symbol="0" user="0"/><hh:ratio hangul="100" latin="100" hanja="100" japanese="100" other="100" symbol="100" user="100"/><hh:spacing hangul="0" latin="0" hanja="0" japanese="0" other="0" symbol="0" user="0"/><hh:relSz hangul="100" latin="100" hanja="100" japanese="100" other="100" symbol="100" user="100"/><hh:offset hangul="0" latin="0" hanja="0" japanese="0" other="0" symbol="0" user="0"/><hh:underline type="NONE" shape="SOLID" color="#000000"/><hh:strikeout shape="NONE" color="#000000"/><hh:outline type="NONE"/><hh:shadow type="NONE" color="#B2B2B2" offsetX="10" offsetY="10"/></hh:charPr></hh:charProperties><hh:tabProperties itemCnt="1"><hh:tabPr id="0" autoTabLeft="0" autoTabRight="0"/></hh:tabProperties>"##);
    s.push_str(&format!(
        r#"<hh:paraProperties itemCnt="{}">"#,
        3 + p.items.len()
    ));
    s.push_str(&para_property(0, "JUSTIFY", 0, false, None));
    s.push_str(&para_property(1, "CENTER", 1100, true, None));
    s.push_str(&para_property(2, "LEFT", 350, true, None));
    for (n, item) in p.items.iter().enumerate() {
        s.push_str(&para_property(
            3 + n as u32,
            "LEFT",
            0,
            false,
            Some(mm(item.answer_space_mm).max(1)),
        ));
    }
    s.push_str(r#"</hh:paraProperties><hh:styles itemCnt="1"><hh:style id="0" type="PARA" name="바탕글" engName="Normal" paraPrIDRef="0" charPrIDRef="0" nextStyleIDRef="0" langID="1042" lockForm="0"/></hh:styles></hh:refList><hh:compatibleDocument targetProgram="HWP201X"><hh:layoutCompatibility/></hh:compatibleDocument><hh:docOption><hh:linkinfo path="" pageInherit="0" footnoteInherit="0"/></hh:docOption></hh:head>"#);
    s
}
fn para_property(id: u32, align: &str, after: u32, keep: bool, fixed: Option<u32>) -> String {
    let (kind, line) = fixed.map(|x| ("FIXED", x)).unwrap_or(("PERCENT", 160));
    format!(
        r#"<hh:paraPr id="{id}" tabPrIDRef="0" condense="0" fontLineHeight="0" snapToGrid="1" suppressLineNumbers="0" checked="0"><hh:align horizontal="{align}" vertical="BASELINE"/><hh:heading type="NONE" idRef="0" level="0"/><hh:breakSetting breakLatinWord="KEEP_WORD" breakNonLatinWord="KEEP_WORD" widowOrphan="0" keepWithNext="{}" keepLines="0" pageBreakBefore="0" lineWrap="BREAK"/><hh:autoSpacing eAsianEng="0" eAsianNum="0"/><hh:margin><hc:intent value="0" unit="HWPUNIT"/><hc:left value="0" unit="HWPUNIT"/><hc:right value="0" unit="HWPUNIT"/><hc:prev value="0" unit="HWPUNIT"/><hc:next value="{after}" unit="HWPUNIT"/></hh:margin><hh:lineSpacing type="{kind}" value="{line}" unit="HWPUNIT"/><hh:border borderFillIDRef="1" offsetLeft="0" offsetRight="0" offsetTop="0" offsetBottom="0" connect="0" ignoreMargin="0"/></hh:paraPr>"#,
        u8::from(keep)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Item;
    use std::io::Read;
    fn project() -> Project {
        Project {
            version: 1,
            title: "문항 & <연습>".into(),
            columns: 2,
            start_number: 7,
            items: vec![Item {
                id: "a".into(),
                points: 3.5,
                answer_space_mm: 35.0,
                blocks: vec![
                    Block::Text {
                        text: "조건 x < 3 & y > 1\n다음 줄".into(),
                    },
                    Block::Equation {
                        script: "{x+1} over {x-1}".into(),
                        source_image: None,
                        source_latex: None,
                        conversion_warnings: vec![],
                    },
                ],
            }],
        }
    }
    fn entry(bytes: &[u8], path: &str) -> String {
        let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        let mut s = String::new();
        zip.by_name(path).unwrap().read_to_string(&mut s).unwrap();
        s
    }
    fn png() -> String {
        let mut data = Cursor::new(Vec::new());
        image::DynamicImage::new_rgb8(4, 2)
            .write_to(&mut data, image::ImageFormat::Png)
            .unwrap();
        format!(
            "data:image/png;base64,{}",
            STANDARD.encode(data.into_inner())
        )
    }
    #[test]
    fn zip_signature_and_all_xml_are_valid() {
        let bytes = export_hwpx(&project()).unwrap();
        let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        {
            let mut first = zip.by_index(0).unwrap();
            assert_eq!(first.name(), "mimetype");
            assert_eq!(first.compression(), CompressionMethod::Stored);
            let mut mime = String::new();
            first.read_to_string(&mut mime).unwrap();
            assert_eq!(mime, "application/hwp+zip");
        }
        for n in 0..zip.len() {
            let mut f = zip.by_index(n).unwrap();
            if f.name().ends_with(".xml") || f.name().ends_with(".hpf") {
                let mut s = String::new();
                f.read_to_string(&mut s).unwrap();
                roxmltree::Document::parse(&s).unwrap_or_else(|e| panic!("{}: {e}", f.name()));
            }
        }
    }
    #[test]
    fn content_is_escaped_and_equations_are_editable_objects() {
        let b = export_hwpx(&project()).unwrap();
        let s = entry(&b, "Contents/section0.xml");
        assert!(s.contains("조건 x &lt; 3 &amp; y &gt; 1<hp:lineBreak/>"));
        let doc = roxmltree::Document::parse(&s).unwrap();
        assert_eq!(
            doc.descendants()
                .find(|n| n.tag_name().name() == "script")
                .unwrap()
                .text(),
            Some("{x+1} over {x-1}")
        );
        assert!(s.contains("<hp:equation"));
        assert!(!s.contains("<hp:pic"));
        assert!(s.contains("7.  [3.5점]"));
        assert!(entry(&b, "Contents/content.hpf").contains("문항 &amp; &lt;연습&gt;"));
    }
    #[test]
    fn image_asset_manifest_and_aspect_ratio_match() {
        let mut p = project();
        p.items[0].blocks.push(Block::Image {
            data_url: png(),
            width_mm: 60.0,
            alt: "그래프 <A>".into(),
        });
        let b = export_hwpx(&p).unwrap();
        let s = entry(&b, "Contents/section0.xml");
        let doc = roxmltree::Document::parse(&s).unwrap();
        let pic = doc
            .descendants()
            .find(|n| n.tag_name().name() == "pic")
            .unwrap();
        let sz = pic
            .children()
            .find(|n| n.tag_name().name() == "sz")
            .unwrap();
        let w: u32 = sz.attribute("width").unwrap().parse().unwrap();
        let h: u32 = sz.attribute("height").unwrap().parse().unwrap();
        assert!(w.abs_diff(h * 2) <= 1);
        assert!(s.contains("binaryItemIDRef=\"image1\""));
        assert!(s.contains("그래프 &lt;A&gt;"));
        let manifest = entry(&b, "Contents/content.hpf");
        assert!(manifest.contains("BinData/image1.png"));
        let mut zip = zip::ZipArchive::new(Cursor::new(b)).unwrap();
        let mut payload = Vec::new();
        zip.by_name("BinData/image1.png")
            .unwrap()
            .read_to_end(&mut payload)
            .unwrap();
        assert!(image::load_from_memory(&payload).is_ok());
    }
    #[test]
    fn columns_and_answer_space_are_real_layout_properties() {
        for columns in [1, 2] {
            let mut p = project();
            p.columns = columns;
            let b = export_hwpx(&p).unwrap();
            assert!(entry(&b, "Contents/section0.xml").contains(&format!("colCount=\"{columns}\"")));
            let h = entry(&b, "Contents/header.xml");
            assert!(h.contains(&format!("type=\"FIXED\" value=\"{}\"", mm(35.0))));
        }
    }
    #[test]
    fn rejects_invalid_and_external_payloads() {
        let mut p = project();
        p.columns = 3;
        assert!(export_hwpx(&p).is_err());
        p.columns = 1;
        p.items[0].points = f64::NAN;
        assert!(export_hwpx(&p).is_err());
        p.items[0].points = 1.0;
        p.title = "bad\u{0}".into();
        assert!(export_hwpx(&p).is_err());
        assert!(decode_asset("https://example.com/image.png", 1).is_err());
        assert!(decode_asset("data:image/png;base64,AAAA", 1).is_err());
    }
    #[test]
    fn serde_contract_round_trips() {
        let p = project();
        let s = serde_json::to_string(&p).unwrap();
        assert!(s.contains("startNumber"));
        assert!(s.contains("answerSpaceMm"));
        let decoded: Project = serde_json::from_str(&s).unwrap();
        assert_eq!(decoded.columns, 2);
        let b: Block =
            serde_json::from_str(r#"{"kind":"image","dataUrl":"abc","widthMm":60,"alt":"a"}"#)
                .unwrap();
        assert!(matches!(b, Block::Image { width_mm: 60.0, .. }));
    }
    #[test]
    fn equations_escape_special_characters_without_losing_script() {
        let mut p = project();
        p.items[0].blocks = vec![Block::Equation {
            script: "x < y & \"한글\"".into(),
            source_image: None,
            source_latex: None,
            conversion_warnings: vec![],
        }];
        let b = export_hwpx(&p).unwrap();
        let xml = entry(&b, "Contents/section0.xml");
        let doc = roxmltree::Document::parse(&xml).unwrap();
        assert_eq!(
            doc.descendants()
                .find(|n| n.tag_name().name() == "script")
                .unwrap()
                .text(),
            Some("x < y & \"한글\"")
        );
    }
    #[test]
    fn manifest_paths_resolve_and_style_references_exist() {
        let p = project();
        let b = export_hwpx(&p).unwrap();
        let manifest = entry(&b, "Contents/content.hpf");
        let doc = roxmltree::Document::parse(&manifest).unwrap();
        let mut zip = zip::ZipArchive::new(Cursor::new(&b)).unwrap();
        for n in doc.descendants().filter(|n| n.tag_name().name() == "item") {
            assert!(zip.by_name(n.attribute("href").unwrap()).is_ok());
        }
        let header = entry(&b, "Contents/header.xml");
        let hd = roxmltree::Document::parse(&header).unwrap();
        let ids: std::collections::HashSet<_> = hd
            .descendants()
            .filter(|n| n.tag_name().name() == "paraPr")
            .map(|n| n.attribute("id").unwrap())
            .collect();
        let section = entry(&b, "Contents/section0.xml");
        let sd = roxmltree::Document::parse(&section).unwrap();
        for n in sd.descendants().filter(|n| n.tag_name().name() == "p") {
            assert!(ids.contains(n.attribute("paraPrIDRef").unwrap()));
        }
    }
    #[test]
    fn image_is_clamped_to_column_width() {
        let mut p = project();
        p.items[0].blocks = vec![Block::Image {
            data_url: png(),
            width_mm: 180.0,
            alt: "figure".into(),
        }];
        let b = export_hwpx(&p).unwrap();
        let s = entry(&b, "Contents/section0.xml");
        let doc = roxmltree::Document::parse(&s).unwrap();
        let pic = doc
            .descendants()
            .find(|n| n.tag_name().name() == "pic")
            .unwrap();
        let sz = pic
            .children()
            .find(|n| n.tag_name().name() == "sz")
            .unwrap();
        assert_eq!(sz.attribute("width").unwrap(), mm(86.0).to_string());
    }

    #[test]
    fn equation_reference_metadata_roundtrips_without_printing_duplicate_image() {
        let mut p = project();
        p.items[0].blocks = vec![Block::Equation {
            script: "x^2".into(),
            source_image: Some(png()),
            source_latex: Some("x^{2}".into()),
            conversion_warnings: vec!["Review".into()],
        }];
        let json = serde_json::to_string(&p).unwrap();
        assert!(json.contains("sourceImage"));
        assert!(json.contains("sourceLatex"));
        let p: Project = serde_json::from_str(&json).unwrap();
        let bytes = export_hwpx(&p).unwrap();
        let section = entry(&bytes, "Contents/section0.xml");
        assert!(section.contains("<hp:equation"));
        assert!(!section.contains("<hp:pic"));
        assert!(!entry(&bytes, "Contents/content.hpf").contains("BinData"));
    }
}
