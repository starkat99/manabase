mod card;
mod scryfall;
mod tags;
mod templates;

#[macro_use]
extern crate log;

use crate::{card::TaggedCardDb, scryfall::CardList};
use clap::{App, Arg};
use fs_extra::dir::{self, CopyOptions};
use std::{fs::File, io::Write, path::Path};

static BULK_DATA_URL: &'static str = "https://archive.scryfall.com/json/scryfall-oracle-cards.json";
static IMAGE_FRONT_BASE_PATH: &'static str = "img/card/front";
static IMAGE_BACK_BASE_PATH: &'static str = "img/card/back";

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

    let output_dir = Path::new(matches.value_of_os("output").unwrap());

    std::fs::create_dir_all(&output_dir)?;
    let copy_opts = CopyOptions {
        overwrite: true,
        ..CopyOptions::new()
    };
    dir::copy("img", output_dir, &copy_opts)?;
    dir::copy("style", output_dir, &copy_opts)?;
    dir::copy("script", output_dir, &copy_opts)?;
    std::fs::create_dir_all(output_dir.join(IMAGE_FRONT_BASE_PATH))?;
    std::fs::create_dir_all(output_dir.join(IMAGE_BACK_BASE_PATH))?;

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

    info!("downloading card images");
    for tagged_card in carddb.cards() {
        let card = tagged_card.card();
        if tagged_card.back_image_path().is_some() {
            let faces = card.card_faces.as_ref().unwrap();
            if !output_dir.join(tagged_card.front_image_path()).exists() {
                let image_uris = faces[0].image_uris.as_ref().unwrap();
                if let Some(uri) = image_uris.get("normal") {
                    trace!(
                        "downloading card '{}' front image from {} to {}",
                        &card.name,
                        &uri,
                        tagged_card.front_image_path().display()
                    );
                    let mut file = File::create(output_dir.join(tagged_card.front_image_path()))?;
                    reqwest::get(uri.as_ref())?.copy_to(&mut file)?;
                    file.flush()?;
                }
            }

            if !output_dir
                .join(tagged_card.back_image_path().unwrap())
                .exists()
            {
                let image_uris = faces[1].image_uris.as_ref().unwrap();
                if let Some(uri) = image_uris.get("normal") {
                    trace!(
                        "downloading card '{}' back image from {} to {}",
                        &card.name,
                        &uri,
                        tagged_card.back_image_path().unwrap().display()
                    );
                    reqwest::get(uri.as_ref())?.copy_to(&mut File::create(
                        output_dir.join(tagged_card.back_image_path().unwrap()),
                    )?)?;
                }
            }
        } else {
            if !output_dir.join(tagged_card.front_image_path()).exists() {
                if let Some(uri) = card.image_uris.as_ref().unwrap().get("normal") {
                    trace!(
                        "downloading card '{}' image from {} to {}",
                        &card.name,
                        &uri,
                        tagged_card.front_image_path().display()
                    );
                    let mut file = File::create(output_dir.join(tagged_card.front_image_path()))?;
                    reqwest::get(uri.as_ref())?.copy_to(&mut file)?;
                    file.flush()?;
                }
            }
        }
    }

    info!("complete");
    Ok(())
}
