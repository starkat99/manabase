use crate::{
    scryfall::{Card, CardList},
    tags::{Category, TagData, TagIndex},
};
use std::collections::{BTreeSet, HashMap, HashSet};

#[derive(Debug)]
pub struct TaggedCardDb<'a, 'b> {
    card_index: HashMap<&'a str, TaggedCard<'a, 'b>>,
    tag_index: HashMap<&'b TagData, HashSet<&'a str>>,
    category_index: HashMap<Category, HashSet<&'a str>>,
}

#[derive(Debug)]
pub struct TaggedCard<'a, 'b> {
    card: &'a Card<'a>,
    tags: HashSet<&'b TagData>,
    categories: BTreeSet<Category>,
    front_image_uri: &'a str,
    back_image_uri: Option<&'a str>,
}

impl<'a, 'b> TaggedCardDb<'a, 'b> {
    pub fn new() -> Self {
        TaggedCardDb {
            card_index: HashMap::default(),
            tag_index: HashMap::default(),
            category_index: HashMap::default(),
        }
    }

    pub fn cards(&self) -> impl Iterator<Item = &TaggedCard<'a, 'b>> {
        self.card_index.values()
    }

    pub fn build(&mut self, tag_index: &'b TagIndex, cards: &'a CardList<'a>) {
        for card in cards.cards() {
            trace!("testing card '{}'", &card.name);
            let mut tags: HashSet<&'b TagData> = HashSet::new();
            let mut categories: BTreeSet<Category> = BTreeSet::new();
            for (_tag, tag_data) in tag_index.iter() {
                for condition in tag_data.iter() {
                    if condition.is_match(&card) {
                        tags.insert(&tag_data);
                        categories.insert(condition.category());
                    }
                }
            }

            // Include all lands, even if untagged
            if card
                .type_line
                .as_ref()
                .filter(|s| Category::Lands.type_regex().unwrap().is_match(&s))
                .is_some()
            {
                categories.insert(Category::Lands);
            }

            if !categories.is_empty() {
                trace!("card '{}' matched", &card.name);
                for category in &categories {
                    if let Some(ids) = self.category_index.get_mut(category) {
                        ids.insert(card.id.as_ref());
                    } else {
                        let mut set = HashSet::new();
                        set.insert(card.id.as_ref());
                        self.category_index.insert(*category, set);
                    }
                }
                for tag in &tags {
                    if let Some(ids) = self.tag_index.get_mut(tag) {
                        ids.insert(card.id.as_ref());
                    } else {
                        let mut set = HashSet::new();
                        set.insert(card.id.as_ref());
                        self.tag_index.insert(tag.clone(), set);
                    }
                }
                let tagged_card = TaggedCard::new(card, tags, categories);
                self.card_index
                    .insert(tagged_card.card.id.as_ref(), tagged_card);
            }
        }
    }
}

impl<'a, 'b> TaggedCard<'a, 'b> {
    fn new(card: &'a Card<'a>, tags: HashSet<&'b TagData>, categories: BTreeSet<Category>) -> Self {
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
            categories,
            front_image_uri,
            back_image_uri,
        }
    }

    pub fn card(&self) -> &Card<'a> {
        &self.card
    }

    pub fn tags(&self) -> Vec<&'b TagData> {
        let mut tags: Vec<_> = self.tags.iter().map(|r| *r).collect();
        tags.sort_by_key(|t| t.name());
        tags
    }

    pub fn categories(&self) -> &BTreeSet<Category> {
        &self.categories
    }

    pub fn front_image_uri(&self) -> &str {
        self.front_image_uri
    }

    pub fn back_image_uri(&self) -> Option<&'a str> {
        self.back_image_uri
    }
}
