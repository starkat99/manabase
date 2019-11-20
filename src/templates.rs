use crate::card::{TaggedCard, TaggedCardDb};
use askama::Template;
use std::{fs::File, io::Write, path::Path};

#[derive(Debug, Template)]
#[template(path = "all.html")]
pub struct AllCards<'a, 'b> {
    cards: Vec<&'a TaggedCard<'a, 'b>>,
}

impl<'a, 'b> AllCards<'a, 'b> {
    pub fn new(carddb: &'a TaggedCardDb<'a, 'b>) -> AllCards<'a, 'b> {
        let mut cards: Vec<_> = carddb.cards().collect();
        debug!("sorting all cards");
        cards.sort_by_key(|c| &c.card().name);
        AllCards { cards }
    }

    pub fn write_output(&self, output_dir: &Path) -> std::io::Result<()> {
        write!(File::create(output_dir.join("all.html"))?, "{}", self)
    }
}
