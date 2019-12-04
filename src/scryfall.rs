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

impl<'a> CardList<'a> {
    pub fn cards(&'a self) -> &'a Vec<Card<'a>> {
        &self.0
    }
}
