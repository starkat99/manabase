use crate::{
    card::{TaggedCard, TaggedCardDb},
    tags::{Category, TagDb},
};
use askama::Template;
use std::{fs::File, io::Write, path::Path};

#[derive(Debug, Template)]
#[template(path = "index.html")]
pub struct IndexPage<'a> {
    categories: [Category; 4],
    tagdb: &'a TagDb<'a>,
}

#[derive(Debug, Template)]
#[template(path = "all.html")]
pub struct AllCards<'a, 'b> {
    title: &'static str,
    cards: Vec<&'a TaggedCard<'a, 'b>>,
}

#[derive(Debug, Template)]
#[template(path = "category.html")]
pub struct CategoryPage<'a> {
    category: Category,
    tagdb: &'a TagDb<'a>,
}

impl<'a> IndexPage<'a> {
    pub fn new(tagdb: &'a TagDb<'a>) -> IndexPage<'a> {
        IndexPage {
            categories: [
                Category::Lands,
                Category::Rocks,
                Category::Dorks,
                Category::Ramp,
            ],
            tagdb,
        }
    }

    pub fn write_output(&self, output_dir: &Path) -> std::io::Result<()> {
        write!(File::create(output_dir.join("index.html"))?, "{}", self)
    }
}

impl<'a, 'b> AllCards<'a, 'b> {
    pub fn new(carddb: &'a TaggedCardDb<'a, 'b>) -> AllCards<'a, 'b> {
        let mut cards: Vec<_> = carddb.cards().collect();
        debug!("sorting all cards");
        cards.sort_unstable_by_key(|c| &c.card().name);
        AllCards {
            title: "All cards",
            cards,
        }
    }

    pub fn write_output(&self, output_dir: &Path) -> std::io::Result<()> {
        write!(File::create(output_dir.join("all.html"))?, "{}", self)
    }
}

impl<'a> CategoryPage<'a> {
    pub fn new(category: Category, tagdb: &'a TagDb<'a>) -> CategoryPage<'a> {
        CategoryPage { category, tagdb }
    }

    pub fn category(&self) -> &Category {
        &self.category
    }

    pub fn write_output(&self, output_dir: &Path) -> std::io::Result<()> {
        write!(
            File::create(output_dir.join(self.category.base_uri()))?,
            "{}",
            self
        )
    }
}
