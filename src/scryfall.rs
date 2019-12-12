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
