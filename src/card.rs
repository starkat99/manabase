use crate::{
    scryfall::{Card, CardList},
    tags::{Category, CategoryList, TagIndex, TagRef},
};
use itertools::join;
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    hash::{Hash, Hasher},
    ops::Deref,
    ptr,
};

#[derive(Debug)]
pub struct TaggedCardDb<'a> {
    card_index: HashMap<CardId<'a>, TaggedCard<'a>>,
    tag_index: HashMap<TagRef<'a>, HashSet<CardId<'a>>>,
    category_index: HashMap<Category, HashSet<CardId<'a>>>,
}

#[derive(Debug)]
pub struct TaggedCard<'a> {
    card: &'a Card<'a>,
    tags: HashSet<TagRef<'a>>,
    categories: BTreeSet<Category>,
    front_image_uri: &'a str,
    back_image_uri: Option<&'a str>,
}

#[derive(Debug)]
pub struct CardId<'a>(&'a str);

impl<'a> TaggedCardDb<'a> {
    pub fn new() -> Self {
        TaggedCardDb {
            card_index: HashMap::default(),
            tag_index: HashMap::default(),
            category_index: HashMap::default(),
        }
    }

    pub fn cards(&self) -> impl Iterator<Item = &TaggedCard<'a>> {
        self.card_index.values()
    }

    pub fn build(
        &mut self,
        category_list: &'a CategoryList,
        tag_index: &'a TagIndex,
        cards: &'a CardList<'a>,
    ) {
        for card in cards.cards() {
            trace!("testing card '{}'", &card.name);
            let mut tags: HashSet<TagRef<'a>> = HashSet::new();
            let mut categories: BTreeSet<Category> = BTreeSet::new();
            for (_tag, tag_ref) in tag_index.iter() {
                for condition in tag_ref.iter() {
                    if condition.is_match(category_list, &card) {
                        tags.insert(tag_ref);
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

            // Include listed category cards
            categories.extend(category_list.get_categories(card.name.as_ref()));

            if !categories.is_empty() {
                trace!("card '{}' matched", &card.name);
                for category in &categories {
                    if let Some(ids) = self.category_index.get_mut(category) {
                        ids.insert(CardId::new(card.id.as_ref()));
                    } else {
                        let mut set = HashSet::new();
                        set.insert(CardId::new(card.id.as_ref()));
                        self.category_index.insert(*category, set);
                    }
                }
                for tag in &tags {
                    if let Some(ids) = self.tag_index.get_mut(tag) {
                        ids.insert(CardId::new(card.id.as_ref()));
                    } else {
                        let mut set = HashSet::new();
                        set.insert(CardId::new(card.id.as_ref()));
                        self.tag_index.insert(tag.clone(), set);
                    }
                }
                let tagged_card = TaggedCard::new(card, tags, categories);
                self.card_index
                    .insert(CardId::new(tagged_card.card.id.as_ref()), tagged_card);
            }
        }
    }

    pub fn card_index(&self) -> &HashMap<CardId<'a>, TaggedCard<'a>> {
        &self.card_index
    }

    pub fn tag_index(&self) -> &HashMap<TagRef<'a>, HashSet<CardId<'a>>> {
        &self.tag_index
    }
}

impl<'a> TaggedCard<'a> {
    fn new(card: &'a Card<'a>, tags: HashSet<TagRef<'a>>, categories: BTreeSet<Category>) -> Self {
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

    pub fn tags(&self) -> Vec<TagRef<'a>> {
        let mut tags: Vec<_> = self.tags.iter().copied().collect();
        tags.sort_unstable_by_key(|&t| t.name().into_owned());
        tags
    }

    pub fn tag_set(&self) -> &HashSet<TagRef<'a>> {
        &self.tags
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

    pub fn has_category(&self, category: &Category) -> bool {
        self.categories.contains(category)
    }

    pub fn category_filter_classes(&self) -> String {
        join(
            self.categories.iter().copied().map(Category::filter_class),
            " ",
        )
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
