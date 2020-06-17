use crate::color::Color;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap};

#[derive(Debug, Serialize, Deserialize)]
pub struct CardList<'a>(#[serde(borrow)] Vec<Card<'a>>);

#[derive(Debug, Serialize, Deserialize)]
pub struct Card<'a> {
    #[serde(borrow)]
    pub id: Cow<'a, str>,
    #[serde(borrow)]
    pub scryfall_uri: Cow<'a, str>,
    #[serde(default)]
    pub card_faces: Option<Vec<CardFace<'a>>>,
    pub cmc: f32,
    pub color_identity: Vec<Color>,
    #[serde(borrow)]
    pub name: Cow<'a, str>,
    #[serde(default, borrow)]
    pub type_line: Option<Cow<'a, str>>,
    #[serde(default, borrow)]
    pub image_uris: Option<HashMap<Cow<'a, str>, Cow<'a, str>>>,
    pub set_type: SetType,
    #[serde(default)]
    pub legalities: Option<HashMap<Format, Legality>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CardFace<'a> {
    #[serde(default, borrow)]
    pub image_uris: Option<HashMap<Cow<'a, str>, Cow<'a, str>>>,
    #[serde(borrow)]
    pub name: Cow<'a, str>,
    #[serde(default, borrow)]
    pub type_line: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SetType {
    Funny,
    Memorabilia,
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Format {
    Standard,
    Historic,
    Pioneer,
    Modern,
    Legacy,
    Vintage,
    Pauper,
    Commander,
    Brawl,
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Legality {
    NotLegal,
    Legal,
    Restricted,
    Banned,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkDataInfo {
    pub download_uri: String,
}

impl<'a> CardList<'a> {
    pub fn cards(&'a self) -> &'a Vec<Card<'a>> {
        &self.0
    }
}

impl SetType {
    pub fn filter_class(self) -> &'static str {
        match self {
            SetType::Other => "",
            _ => "mtg-filter-silver-border",
        }
    }
}

impl std::fmt::Display for Format {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Format::*;
        write!(
            fmt,
            "{}",
            match self {
                Standard => "Standard",
                Historic => "Historic",
                Pioneer => "Pioneer",
                Modern => "Modern",
                Legacy => "Legacy",
                Vintage => "Vintage",
                Pauper => "Pauper",
                Commander => "Commander",
                Brawl => "Brawl",
                Other => "Other",
            }
        )
    }
}

impl std::fmt::Display for Legality {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Legality::*;
        write!(
            fmt,
            "{}",
            match self {
                NotLegal => "Not Legal",
                Legal => "Legal",
                Restricted => "Restricted",
                Banned => "Banned",
            }
        )
    }
}
