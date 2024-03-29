use crate::{
    card::{CardType, TaggedCard, TaggedCardDb},
    tags::{TagDb, TagIndex, TagRef},
};
use askama::Template;
use chrono::prelude::*;
use log::debug;
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Write,
    path::Path,
};

#[derive(Debug, Template)]
#[template(path = "index.html")]
pub struct IndexPage<'a> {
    card_types: [CardType; 8],
    tagdb: &'a TagDb<'a>,
    carddb: &'a TaggedCardDb<'a>,
    timestamp: DateTime<Utc>,
    data_updated: DateTime<Utc>,
}

#[derive(Debug, Template)]
#[template(path = "all.html")]
pub struct AllCards<'a> {
    cards: Vec<&'a TaggedCard<'a>>,
    timestamp: DateTime<Utc>,
    data_updated: DateTime<Utc>,
}

#[derive(Debug, Template)]
#[template(path = "type-all.html")]
pub struct TypeAllCards<'a> {
    card_type: CardType,
    cards: Vec<&'a TaggedCard<'a>>,
    timestamp: DateTime<Utc>,
    data_updated: DateTime<Utc>,
}

#[derive(Debug, Template)]
#[template(path = "card_type.html")]
pub struct TypePage<'a> {
    card_type: CardType,
    tagdb: &'a TagDb<'a>,
    carddb: &'a TaggedCardDb<'a>,
    timestamp: DateTime<Utc>,
    data_updated: DateTime<Utc>,
}

#[derive(Debug, Template)]
#[template(path = "tag-cards.html")]
pub struct TagPage<'a> {
    card_types: [CardType; 8],
    tag: TagRef<'a>,
    tag_index: &'a TagIndex,
    cards: HashMap<Option<TagRef<'a>>, Vec<&'a TaggedCard<'a>>>,
    carddb: &'a TaggedCardDb<'a>,
    timestamp: DateTime<Utc>,
    data_updated: DateTime<Utc>,
}

impl<'a> IndexPage<'a> {
    pub fn new(
        tagdb: &'a TagDb<'a>,
        carddb: &'a TaggedCardDb<'a>,
        timestamp: DateTime<Utc>,
        data_updated: DateTime<Utc>,
    ) -> IndexPage<'a> {
        IndexPage {
            card_types: [
                CardType::Land,
                CardType::Artifact,
                CardType::Creature,
                CardType::Enchantment,
                CardType::Instant,
                CardType::Sorcery,
                CardType::Planeswalker,
                CardType::Battle,
            ],
            tagdb,
            carddb,
            timestamp,
            data_updated,
        }
    }

    pub fn write_output(&self, output_dir: &Path) -> std::io::Result<()> {
        write!(File::create(output_dir.join("index.html"))?, "{}", self)
    }
}

impl<'a> AllCards<'a> {
    pub fn new(
        carddb: &'a TaggedCardDb<'a>,
        timestamp: DateTime<Utc>,
        data_updated: DateTime<Utc>,
    ) -> AllCards<'a> {
        let mut cards: Vec<_> = carddb.cards().collect();
        debug!("sorting all cards");
        cards.sort_unstable_by_key(|c| &c.card().name);
        AllCards {
            cards,
            timestamp,
            data_updated,
        }
    }

    pub fn write_output(&self, output_dir: &Path) -> std::io::Result<()> {
        write!(File::create(output_dir.join("all.html"))?, "{}", self)
    }
}

impl<'a> TypeAllCards<'a> {
    pub fn new(
        card_type: CardType,
        carddb: &'a TaggedCardDb<'a>,
        timestamp: DateTime<Utc>,
        data_updated: DateTime<Utc>,
    ) -> TypeAllCards<'a> {
        let mut cards: Vec<_> = carddb
            .cards()
            .filter(|c| c.types().contains(&card_type))
            .collect();
        debug!("sorting all cards");
        cards.sort_unstable_by_key(|c| &c.card().name);
        TypeAllCards {
            card_type,
            cards,
            timestamp,
            data_updated,
        }
    }

    pub fn write_output(&self, output_dir: &Path) -> std::io::Result<()> {
        write!(
            File::create(output_dir.join(self.card_type.all_base_uri()))?,
            "{}",
            self
        )
    }
}

impl<'a> TypePage<'a> {
    pub fn new(
        card_type: CardType,
        tagdb: &'a TagDb<'a>,
        carddb: &'a TaggedCardDb<'a>,
        timestamp: DateTime<Utc>,
        data_updated: DateTime<Utc>,
    ) -> TypePage<'a> {
        TypePage {
            card_type,
            tagdb,
            carddb,
            timestamp,
            data_updated,
        }
    }

    pub fn write_output(&self, output_dir: &Path) -> std::io::Result<()> {
        write!(
            File::create(output_dir.join(self.card_type.base_uri()))?,
            "{}",
            self
        )
    }
}

impl<'a> TagPage<'a> {
    pub fn new(
        tag: TagRef<'a>,
        tag_index: &'a TagIndex,
        carddb: &'a TaggedCardDb<'a>,
        timestamp: DateTime<Utc>,
        data_updated: DateTime<Utc>,
    ) -> TagPage<'a> {
        let subtags: HashSet<_> = tag
            .subtags()
            .iter()
            .filter_map(|name| tag_index.get(name))
            .collect();
        let cards: Vec<_> = carddb
            .tag_index()
            .get(&tag)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| carddb.card_index().get(id))
                    .collect()
            })
            .unwrap_or_default();
        let mut tag_map: HashMap<_, Vec<_>> = HashMap::new();
        for subtag in subtags.iter() {
            let mut tagcards: Vec<_> = cards
                .iter()
                .map(|card| *card)
                .filter(|card| card.tag_set().contains(subtag))
                .collect();
            tagcards.sort_unstable_by_key(|card| &card.card().name);
            tag_map.insert(Some(*subtag), tagcards);
        }
        let mut untagged: Vec<_> = cards
            .iter()
            .map(|card| *card)
            .filter(|card| card.tag_set().is_disjoint(&subtags))
            .collect();
        untagged.sort_unstable_by_key(|card| &card.card().name);
        if !untagged.is_empty() {
            tag_map.insert(None, untagged);
        }
        TagPage {
            card_types: [
                CardType::Land,
                CardType::Artifact,
                CardType::Creature,
                CardType::Enchantment,
                CardType::Instant,
                CardType::Sorcery,
                CardType::Planeswalker,
                CardType::Battle,
            ],
            tag,
            tag_index,
            cards: tag_map,
            carddb,
            timestamp,
            data_updated,
        }
    }

    pub fn tag_ref(&self) -> &TagRef<'a> {
        &self.tag
    }

    pub fn subtags(&self) -> Vec<TagRef<'a>> {
        self.tag
            .subtags()
            .iter()
            .filter_map(|name| self.tag_index.get(name))
            .collect()
    }

    pub fn get_tag_cards(&self, tag: TagRef<'a>) -> Option<&Vec<&'a TaggedCard<'a>>> {
        self.cards.get(&Some(tag.clone()))
    }

    pub fn get_untagged_cards(&self) -> Option<&Vec<&'a TaggedCard<'a>>> {
        self.cards.get(&None)
    }

    pub fn subtag_has_cards_of_type(&self, tag: &TagRef<'a>, card_type: &CardType) -> bool {
        self.get_tag_cards(*tag)
            .map(|vec| vec.iter().any(|card| card.has_type(card_type)))
            .unwrap_or_default()
    }

    pub fn has_untagged_cards_of_type(&self, card_type: &CardType) -> bool {
        self.get_untagged_cards()
            .map(|vec| vec.iter().any(|card| card.has_type(card_type)))
            .unwrap_or_default()
    }

    pub fn write_output(&self, output_dir: &Path) -> std::io::Result<()> {
        write!(
            File::create(output_dir.join(format!("tag-{}.html", self.tag.canonical_name())))?,
            "{}",
            self
        )
    }
}
