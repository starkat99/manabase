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
    pub card_faces: Option<Vec<CardFace<'a>>>,
    pub cmc: f32,
    pub color_identity: Vec<Color>,
    #[serde(borrow)]
    pub name: Cow<'a, str>,
    #[serde(borrow)]
    pub oracle_text: Option<Cow<'a, str>>,
    #[serde(default, borrow)]
    pub type_line: Option<Cow<'a, str>>,
    #[serde(borrow)]
    pub image_uris: Option<HashMap<Cow<'a, str>, Cow<'a, str>>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CardFace<'a> {
    #[serde(borrow)]
    pub image_uris: Option<HashMap<Cow<'a, str>, Cow<'a, str>>>,
    #[serde(borrow)]
    pub name: Cow<'a, str>,
    #[serde(borrow)]
    pub oracle_text: Option<Cow<'a, str>>,
    #[serde(borrow)]
    pub type_line: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Color {
    #[serde(rename = "W")]
    White,
    #[serde(rename = "U")]
    Blue,
    #[serde(rename = "B")]
    Black,
    #[serde(rename = "R")]
    Red,
    #[serde(rename = "G")]
    Green,
}

impl<'a> CardList<'a> {
    pub fn cards(&'a self) -> &'a Vec<Card<'a>> {
        &self.0
    }
}

impl Color {
    pub fn mana_symbol(self) -> &'static str {
        match self {
            Color::White => "<span class=\"mana sw\"></span>",
            Color::Blue => "<span class=\"mana su\"></span>",
            Color::Black => "<span class=\"mana sb\"></span>",
            Color::Red => "<span class=\"mana sr\"></span>",
            Color::Green => "<span class=\"mana sg\"></span>",
        }
    }
}
