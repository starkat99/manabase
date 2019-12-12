use crate::{
    scryfall::{Card, CardList},
    tags::{CardTags, TagIndex, TagRef},
};
use itertools::join;
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    hash::{Hash, Hasher},
    ops::Deref,
    ptr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CardType {
    Land,
    Artifact,
    Creature,
    Enchantment,
    Instant,
    Sorcery,
    Planeswalker,
}

#[derive(Debug)]
pub struct TaggedCardDb<'a> {
    card_index: HashMap<CardId<'a>, TaggedCard<'a>>,
    tag_index: HashMap<TagRef<'a>, HashSet<CardId<'a>>>,
    type_tag_index: HashMap<CardType, HashSet<TagRef<'a>>>,
}

#[derive(Debug)]
pub struct TaggedCard<'a> {
    card: &'a Card<'a>,
    tags: HashSet<TagRef<'a>>,
    types: BTreeSet<CardType>,
    front_image_uri: &'a str,
    back_image_uri: Option<&'a str>,
}

#[derive(Debug)]
pub struct CardId<'a>(&'a str);

impl CardType {
    fn from_str(type_line: &str) -> BTreeSet<CardType> {
        let mut types = BTreeSet::new();
        if type_line.contains("Land") {
            types.insert(CardType::Land);
        }
        if type_line.contains("Artifact") {
            types.insert(CardType::Artifact);
        }
        if type_line.contains("Creature") {
            types.insert(CardType::Creature);
        }
        if type_line.contains("Enchantment") {
            types.insert(CardType::Enchantment);
        }
        if type_line.contains("Instant") {
            types.insert(CardType::Instant);
        }
        if type_line.contains("Sorcery") {
            types.insert(CardType::Sorcery);
        }
        if type_line.contains("Planeswalker") {
            types.insert(CardType::Planeswalker);
        }
        types
    }

    pub fn base_uri(self) -> &'static str {
        match self {
            CardType::Land => "lands.html",
            CardType::Artifact => "artifacts.html",
            CardType::Creature => "creatures.html",
            CardType::Enchantment => "enchantments.html",
            CardType::Instant => "instants.html",
            CardType::Sorcery => "sorceries.html",
            CardType::Planeswalker => "planeswalkers.html",
        }
    }

    pub fn filter_class(self) -> &'static str {
        match self {
            CardType::Land => "mtg-filter-land",
            CardType::Artifact => "mtg-filter-artifact",
            CardType::Creature => "mtg-filter-creature",
            CardType::Enchantment => "mtg-filter-enchantment",
            CardType::Instant => "mtg-filter-instant",
            CardType::Sorcery => "mtg-filter-sorcery",
            CardType::Planeswalker => "mtg-filter-planeswalker",
        }
    }

    pub fn show_only_filter_query(self) -> &'static str {
        match self {
            CardType::Land => "show=land&amp;hide=artifact&amp;hide=creature&amp;hide=enchantment&amp;hide=instant&amp;hide=sorcery&amp;hide=planeswalker",
            CardType::Artifact => "show=artifact&amp;hide=land&amp;hide=creature&amp;hide=enchantment&amp;hide=instant&amp;hide=sorcery&amp;hide=planeswalker",
            CardType::Creature => "show=creature&amp;hide=land&amp;hide=artifact&amp;hide=enchantment&amp;hide=instant&amp;hide=sorcery&amp;hide=planeswalker",
            CardType::Enchantment => "show=enchantment&amp;hide=land&amp;hide=artifact&amp;hide=creature&amp;hide=instant&amp;hide=sorcery&amp;hide=planeswalker",
            CardType::Instant => "show=instant&amp;hide=land&amp;hide=artifact&amp;hide=creature&amp;hide=enchantment&amp;hide=sorcery&amp;hide=planeswalker",
            CardType::Sorcery => "show=sorcery&amp;hide=land&amp;hide=artifact&amp;hide=creature&amp;hide=enchantment&amp;hide=instant&amp;hide=planeswalker",
            CardType::Planeswalker => "show=planeswalker&amp;hide=land&amp;hide=artifact&amp;hide=creature&amp;hide=enchantment&amp;hide=instant&amp;hide=sorcery",
        }
    }
}

impl<'a> TaggedCardDb<'a> {
    pub fn new(card_tags: &'a CardTags, tag_index: &'a TagIndex, cards: &'a CardList<'a>) -> Self {
        let mut card_index: HashMap<CardId<'a>, TaggedCard<'a>> = HashMap::new();
        let mut card_tag_index: HashMap<TagRef<'a>, HashSet<CardId<'a>>> = HashMap::new();
        let mut type_tag_index: HashMap<CardType, HashSet<TagRef<'a>>> = HashMap::new();

        type_tag_index.insert(CardType::Land, HashSet::new());
        type_tag_index.insert(CardType::Artifact, HashSet::new());
        type_tag_index.insert(CardType::Creature, HashSet::new());
        type_tag_index.insert(CardType::Enchantment, HashSet::new());
        type_tag_index.insert(CardType::Instant, HashSet::new());
        type_tag_index.insert(CardType::Sorcery, HashSet::new());
        type_tag_index.insert(CardType::Planeswalker, HashSet::new());

        for (tags, card) in cards
            .cards()
            .iter()
            .filter_map(|card| card_tags.get_tags(&card.name).map(|tags| (tags, card)))
        {
            trace!("tagging card '{}'", &card.name);
            let type_line = join(
                card.type_line.iter().chain(
                    card.card_faces
                        .iter()
                        .flatten()
                        .flat_map(|face| face.type_line.iter()),
                ),
                " ",
            );
            let types = CardType::from_str(&type_line);

            let mut tags: HashSet<_> = tags
                .iter()
                .map(|tag| tag_index.get(&tag).expect(&format!("invalid tag {}", &tag)))
                .collect();
            for (_, tag_ref) in tag_index.iter() {
                if tag_ref.is_match(&card, &type_line) {
                    tags.insert(tag_ref);
                }
            }

            for tag in &tags {
                if let Some(ids) = card_tag_index.get_mut(tag) {
                    ids.insert(CardId::new(card.id.as_ref()));
                } else {
                    let mut set = HashSet::new();
                    set.insert(CardId::new(card.id.as_ref()));
                    card_tag_index.insert(tag.clone(), set);
                }

                for card_type in &types {
                    type_tag_index
                        .get_mut(card_type)
                        .unwrap()
                        .insert(tag.clone());
                }
            }
            let tagged_card = TaggedCard::new(card, tags, types);
            card_index.insert(CardId::new(tagged_card.card.id.as_ref()), tagged_card);
        }
        TaggedCardDb {
            card_index,
            tag_index: card_tag_index,
            type_tag_index,
        }
    }

    pub fn cards(&self) -> impl Iterator<Item = &TaggedCard<'a>> {
        self.card_index.values()
    }

    pub fn card_index(&self) -> &HashMap<CardId<'a>, TaggedCard<'a>> {
        &self.card_index
    }

    pub fn tag_index(&self) -> &HashMap<TagRef<'a>, HashSet<CardId<'a>>> {
        &self.tag_index
    }

    pub fn type_has_cards_of_tag(&self, card_type: CardType, tag: &'a TagRef<'a>) -> bool {
        self.type_tag_index[&card_type].contains(tag)
    }
}

impl<'a> TaggedCard<'a> {
    fn new(card: &'a Card<'a>, tags: HashSet<TagRef<'a>>, types: BTreeSet<CardType>) -> Self {
        let back_image_uri = card
            .card_faces
            .as_ref()
            .and_then(|v| v.get(1))
            .and_then(|f| f.image_uris.as_ref())
            .filter(|_| card.image_uris.is_none())
            .and_then(|m| m.get("normal"))
            .map(|c| c.as_ref());
        let front_image_uri = if back_image_uri.is_some() {
            card.card_faces
                .as_ref()
                .unwrap()
                .get(0)
                .and_then(|f| f.image_uris.as_ref())
                .and_then(|m| m.get("normal"))
                .map(|c| c.as_ref())
                .unwrap_or("")
        } else {
            card.image_uris
                .as_ref()
                .and_then(|m| m.get("normal"))
                .map(|c| c.as_ref())
                .unwrap_or("")
        };
        TaggedCard {
            card,
            tags,
            types,
            front_image_uri,
            back_image_uri,
        }
    }

    pub fn card(&self) -> &Card<'a> {
        &self.card
    }

    pub fn tags(&self) -> Vec<TagRef<'a>> {
        let mut tags: Vec<_> = self.tags.iter().copied().collect();
        tags.sort_unstable_by_key(|&t| t.name().into_owned());
        tags
    }

    pub fn tag_set(&self) -> &HashSet<TagRef<'a>> {
        &self.tags
    }

    pub fn types(&self) -> &BTreeSet<CardType> {
        &self.types
    }

    pub fn front_image_uri(&self) -> &str {
        self.front_image_uri
    }

    pub fn back_image_uri(&self) -> Option<&'a str> {
        self.back_image_uri
    }

    pub fn has_type(&self, card_type: &CardType) -> bool {
        self.types.contains(card_type)
    }

    pub fn type_filter_classes(&self) -> String {
        join(self.types.iter().copied().map(CardType::filter_class), " ")
    }
}

impl<'a> CardId<'a> {
    pub fn new(tag: &'a str) -> CardId<'a> {
        CardId(tag)
    }
}

impl<'a> Clone for CardId<'a> {
    fn clone(&self) -> CardId<'a> {
        CardId(self.0)
    }
}

impl<'a> Copy for CardId<'a> {}

impl<'a> PartialEq for CardId<'a> {
    fn eq(&self, other: &CardId) -> bool {
        ptr::eq(self.0, other.0)
    }
}

impl<'a> Eq for CardId<'a> {}

impl<'a> Hash for CardId<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ptr::hash(self.0, state)
    }
}

impl<'a> AsRef<str> for CardId<'a> {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl<'a> Deref for CardId<'a> {
    type Target = str;
    fn deref(&self) -> &str {
        self.0
    }
}

impl std::fmt::Display for CardType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            fmt,
            "{}",
            match self {
                CardType::Land => "Land",
                CardType::Artifact => "Artifact",
                CardType::Creature => "Creature",
                CardType::Enchantment => "Enchantment",
                CardType::Instant => "Instant",
                CardType::Sorcery => "Sorcery",
                CardType::Planeswalker => "Planeswalker",
            }
        )
    }
}
