mod card;
mod color;
mod scryfall;
mod tags;
mod templates;

#[macro_use]
extern crate log;

use crate::{
    card::{CardType, TaggedCardDb},
    scryfall::{BulkDataInfo, CardList},
    tags::{CardTags, TagDb, TagIndex},
};
use chrono::prelude::*;
use clap::{App, Arg};
use fs_extra::dir::{self, CopyOptions};
use std::{collections::HashSet, path::Path};

static BULK_DATA_API_URL: &'static str = "https://api.scryfall.com/bulk-data/oracle-cards";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
    let config_dir = &Path::new("config");
    let mut tag_index = TagIndex::load(&config_dir.join("tags.toml"))?;
    let card_tags = CardTags::load(&config_dir.join("card-tags.toml"))?;
    tag_index.merge_tags(&card_tags);
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
    let data_updated: DateTime<Utc>;
    if let Some(path) = matches.value_of_os("data") {
        let path = Path::new(path);
        info!("loading Scryfall bulk card data from {}", path.display());
        bulk_data = std::fs::read_to_string(path)?;
        data_updated = std::fs::metadata(path)?.modified()?.into();
    } else {
        let bulk_data_info = reqwest::get(BULK_DATA_API_URL)
            .await?
            .json::<BulkDataInfo>()
            .await?;

        info!(
            "loading Scryfall bulk card data from {}",
            &bulk_data_info.download_uri
        );
        bulk_data = reqwest::get(&bulk_data_info.download_uri)
            .await?
            .text()
            .await?;
        data_updated = bulk_data_info.updated_at;
    }
    let timestamp = Utc::now();
    let cards: CardList = serde_json::from_str(&bulk_data)?;
    debug!("loaded {} cards", cards.cards().len());

    info!("tagging cards");
    let carddb = TaggedCardDb::new(&card_tags, &tag_index, &cards);

    info!("creating template pages");
    templates::IndexPage::new(&tagdb, &carddb, timestamp.clone(), data_updated.clone())
        .write_output(&output_dir)?;
    debug!("writing all cards page");
    templates::AllCards::new(&carddb, timestamp.clone(), data_updated.clone())
        .write_output(&output_dir)?;
    debug!("writing all type card pages");
    templates::TypeAllCards::new(
        CardType::Land,
        &carddb,
        timestamp.clone(),
        data_updated.clone(),
    )
    .write_output(&output_dir)?;
    templates::TypeAllCards::new(
        CardType::Artifact,
        &carddb,
        timestamp.clone(),
        data_updated.clone(),
    )
    .write_output(&output_dir)?;
    templates::TypeAllCards::new(
        CardType::Creature,
        &carddb,
        timestamp.clone(),
        data_updated.clone(),
    )
    .write_output(&output_dir)?;
    templates::TypeAllCards::new(
        CardType::Enchantment,
        &carddb,
        timestamp.clone(),
        data_updated.clone(),
    )
    .write_output(&output_dir)?;
    templates::TypeAllCards::new(
        CardType::Instant,
        &carddb,
        timestamp.clone(),
        data_updated.clone(),
    )
    .write_output(&output_dir)?;
    templates::TypeAllCards::new(
        CardType::Sorcery,
        &carddb,
        timestamp.clone(),
        data_updated.clone(),
    )
    .write_output(&output_dir)?;
    templates::TypeAllCards::new(
        CardType::Planeswalker,
        &carddb,
        timestamp.clone(),
        data_updated.clone(),
    )
    .write_output(&output_dir)?;
    debug!("writing card type pages");
    templates::TypePage::new(
        CardType::Land,
        &tagdb,
        &carddb,
        timestamp.clone(),
        data_updated.clone(),
    )
    .write_output(&output_dir)?;
    templates::TypePage::new(
        CardType::Artifact,
        &tagdb,
        &carddb,
        timestamp.clone(),
        data_updated.clone(),
    )
    .write_output(&output_dir)?;
    templates::TypePage::new(
        CardType::Creature,
        &tagdb,
        &carddb,
        timestamp.clone(),
        data_updated.clone(),
    )
    .write_output(&output_dir)?;
    templates::TypePage::new(
        CardType::Enchantment,
        &tagdb,
        &carddb,
        timestamp.clone(),
        data_updated.clone(),
    )
    .write_output(&output_dir)?;
    templates::TypePage::new(
        CardType::Instant,
        &tagdb,
        &carddb,
        timestamp.clone(),
        data_updated.clone(),
    )
    .write_output(&output_dir)?;
    templates::TypePage::new(
        CardType::Sorcery,
        &tagdb,
        &carddb,
        timestamp.clone(),
        data_updated.clone(),
    )
    .write_output(&output_dir)?;
    templates::TypePage::new(
        CardType::Planeswalker,
        &tagdb,
        &carddb,
        timestamp.clone(),
        data_updated.clone(),
    )
    .write_output(&output_dir)?;
    debug!("writing tag pages");
    for (_, tag) in tag_index.iter() {
        templates::TagPage::new(
            tag,
            &tag_index,
            &carddb,
            timestamp.clone(),
            data_updated.clone(),
        )
        .write_output(&output_dir)?;
    }

    debug!("checking for invalid cards");
    let cardset: HashSet<_> = cards.cards().iter().map(|c| c.name.as_ref()).collect();
    for card in card_tags.cards() {
        if !cardset.contains(card) {
            warn!("card \"{}\" not found in database", card);
        }
    }

    info!("complete");
    Ok(())
}
