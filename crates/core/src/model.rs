//! Portable, versioned worksheet format. Equation scripts use Hancom syntax.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Project {
    pub version: u32,
    pub title: String,
    pub columns: u8,
    pub start_number: u32,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Item {
    pub id: String,
    pub points: f64,
    pub answer_space_mm: f64,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum Block {
    Text {
        text: String,
    },
    Equation {
        script: String,
        #[serde(
            default,
            rename = "sourceImage",
            skip_serializing_if = "Option::is_none"
        )]
        source_image: Option<String>,
        #[serde(
            default,
            rename = "sourceLatex",
            skip_serializing_if = "Option::is_none"
        )]
        source_latex: Option<String>,
        #[serde(
            default,
            rename = "conversionWarnings",
            skip_serializing_if = "Vec::is_empty"
        )]
        conversion_warnings: Vec<String>,
    },
    Image {
        #[serde(rename = "dataUrl")]
        data_url: String,
        #[serde(rename = "widthMm")]
        width_mm: f64,
        alt: String,
    },
}
