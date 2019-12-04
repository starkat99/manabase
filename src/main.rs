mod card;
mod color;
mod scryfall;
mod tags;
mod templates;

#[macro_use]
extern crate log;

use crate::{
    card::TaggedCardDb,
    scryfall::CardList,
    tags::{Category, TagDb},
};
use clap::{App, Arg};
use fs_extra::dir::{self, CopyOptions};
use std::path::Path;

static BULK_DATA_URL: &'static str = "https://archive.scryfall.com/json/scryfall-oracle-cards.json";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    {
        let mut builder = env_logger::Builder::from_default_env();
        builder.target(env_logger::Target::Stdout);
        builder.init();
    }

    let matches = App::new("manabase")
        .arg(
            Arg::with_name("data")
                .short("d")
                .long("data")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .index(1)
                .default_value("target/www"),
        )
        .get_matches();

    info!("loading config files");
    let tag_index = tags::load_tags(&Path::new("config"))?;
    let tagdb = TagDb::new(&tag_index);

    let output_dir = Path::new(matches.value_of_os("output").unwrap());

    std::fs::create_dir_all(&output_dir)?;
    let copy_opts = CopyOptions {
        overwrite: true,
        ..CopyOptions::new()
    };
    dir::copy("img", output_dir, &copy_opts)?;
    dir::copy("style", output_dir, &copy_opts)?;
    dir::copy("script", output_dir, &copy_opts)?;

    let bulk_data: String;
    if let Some(path) = matches.value_of_os("data") {
        let path = Path::new(path);
        info!("loading Scryfall bulk card data from {}", path.display());
        bulk_data = std::fs::read_to_string(path)?;
    } else {
        info!("loading Scryfall bulk card data from {}", BULK_DATA_URL);
        bulk_data = reqwest::get(BULK_DATA_URL)?.text()?;
    }
    let cards: CardList = serde_json::from_str(&bulk_data)?;
    debug!("loaded {} cards", cards.cards().len());

    info!("tagging cards");
    let mut carddb = TaggedCardDb::new();
    carddb.build(&tag_index, &cards);

    info!("creating template pages");
    debug!("writing all cards page");
    templates::AllCards::new(&carddb).write_output(&output_dir)?;
    debug!("writing category pages");
    templates::CategoryPage::new(Category::Lands, &tagdb).write_output(&output_dir)?;
    templates::CategoryPage::new(Category::Rocks, &tagdb).write_output(&output_dir)?;
    templates::CategoryPage::new(Category::Dorks, &tagdb).write_output(&output_dir)?;
    templates::CategoryPage::new(Category::Ramp, &tagdb).write_output(&output_dir)?;

    info!("complete");
    Ok(())
}
