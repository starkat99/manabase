use crate::{
    card::{CardType, TaggedCard, TaggedCardDb},
    tags::{TagDb, TagIndex, TagRef},
};
use askama::Template;
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Write,
    path::Path,
};

#[derive(Debug, Template)]
#[template(path = "index.html")]
pub struct IndexPage<'a> {
    card_types: [CardType; 7],
    tagdb: &'a TagDb<'a>,
    carddb: &'a TaggedCardDb<'a>,
}

#[derive(Debug, Template)]
#[template(path = "all.html")]
pub struct AllCards<'a> {
    cards: Vec<&'a TaggedCard<'a>>,
}

#[derive(Debug, Template)]
#[template(path = "card_type.html")]
pub struct TypePage<'a> {
    card_type: CardType,
    tagdb: &'a TagDb<'a>,
    carddb: &'a TaggedCardDb<'a>,
}

#[derive(Debug, Template)]
#[template(path = "tag-cards.html")]
pub struct TagPage<'a> {
    card_types: [CardType; 7],
    tag: TagRef<'a>,
    tag_index: &'a TagIndex,
    cards: HashMap<Option<TagRef<'a>>, Vec<&'a TaggedCard<'a>>>,
    carddb: &'a TaggedCardDb<'a>,
}

impl<'a> IndexPage<'a> {
    pub fn new(tagdb: &'a TagDb<'a>, carddb: &'a TaggedCardDb<'a>) -> IndexPage<'a> {
        IndexPage {
            card_types: [
                CardType::Land,
                CardType::Artifact,
                CardType::Creature,
                CardType::Enchantment,
                CardType::Instant,
                CardType::Sorcery,
                CardType::Planeswalker,
            ],
            tagdb,
            carddb,
        }
    }

    pub fn write_output(&self, output_dir: &Path) -> std::io::Result<()> {
        write!(File::create(output_dir.join("index.html"))?, "{}", self)
    }
}

impl<'a> AllCards<'a> {
    pub fn new(carddb: &'a TaggedCardDb<'a>) -> AllCards<'a> {
        let mut cards: Vec<_> = carddb.cards().collect();
        debug!("sorting all cards");
        cards.sort_unstable_by_key(|c| &c.card().name);
        AllCards { cards }
    }

    pub fn write_output(&self, output_dir: &Path) -> std::io::Result<()> {
        write!(File::create(output_dir.join("all.html"))?, "{}", self)
    }
}

impl<'a> TypePage<'a> {
    pub fn new(
        card_type: CardType,
        tagdb: &'a TagDb<'a>,
        carddb: &'a TaggedCardDb<'a>,
    ) -> TypePage<'a> {
        TypePage {
            card_type,
            tagdb,
            carddb,
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
            ],
            tag,
            tag_index,
            cards: tag_map,
            carddb,
        }
    }

    pub fn subtags(&self) -> Vec<TagRef<'a>> {
        self.tag
            .subtags()
            .iter()
            .filter_map(|name| self.tag_index.get(name))
            .collect()
    }

    pub fn get_tag_cards(&self, tag: TagRef<'a>) -> Option<&Vec<&'a TaggedCard<'a>>> {
        self.cards.get(&Some(tag))
    }

    pub fn get_untagged_cards(&self) -> Option<&Vec<&'a TaggedCard<'a>>> {
        self.cards.get(&None)
    }

    pub fn subtag_has_cards_of_type(&self, tag: TagRef<'a>, card_type: &CardType) -> bool {
        self.get_tag_cards(tag)
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
