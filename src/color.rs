use crate::scryfall::Color;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ColorMap(HashMap<String, ColorInfo>);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ColorInfo {
    pub colors: Vec<Color>,
    #[serde(rename = "type")]
    pub color_type: ColorType,
    pub name: String,
    pub subtype: Option<ColorSubType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorType {
    Colorless,
    Mono,
    Dual,
    Tri,
    Four,
    Domain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorSubType {
    Allied,
    Enemy,
    Shard,
    Wedge,
}
