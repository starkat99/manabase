use crate::{
    scryfall::{Card, CardList},
    tags::{TagCategory, TagData, TagIndex},
    IMAGE_BACK_BASE_PATH, IMAGE_FRONT_BASE_PATH,
};
use std::{
    borrow::Cow,
    collections::{BTreeSet, HashMap, HashSet},
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct TaggedCardDb<'a, 'b> {
    card_index: HashMap<&'a str, TaggedCard<'a, 'b>>,
    tag_index: HashMap<&'b TagData, HashSet<&'a str>>,
    category_index: HashMap<TagCategory, HashSet<&'a str>>,
}

#[derive(Debug)]
pub struct TaggedCard<'a, 'b> {
    card: &'a Card<'a>,
    tags: HashSet<&'b TagData>,
    categories: BTreeSet<TagCategory>,
    front_image_path: PathBuf,
    back_image_path: Option<PathBuf>,
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
            let mut categories: BTreeSet<TagCategory> = BTreeSet::new();
            for (tag, tag_data) in tag_index.iter() {
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
                .filter(|s| TagCategory::Lands.type_regex().is_match(&s))
                .is_some()
            {
                categories.insert(TagCategory::Lands);
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
    fn new(
        card: &'a Card<'a>,
        tags: HashSet<&'b TagData>,
        categories: BTreeSet<TagCategory>,
    ) -> Self {
        let front_image_path = Path::new(IMAGE_FRONT_BASE_PATH)
            .join(card.id.as_ref())
            .with_extension("jpg");
        let back_image_path = card
            .card_faces
            .as_ref()
            .and_then(|v| v.get(1))
            .and_then(|f| f.image_uris.as_ref())
            .filter(|_| card.image_uris.is_none())
            .map(|_| {
                Path::new(IMAGE_BACK_BASE_PATH)
                    .join(card.id.as_ref())
                    .with_extension("jpg")
            });
        TaggedCard {
            card,
            tags,
            categories,
            front_image_path,
            back_image_path,
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

    pub fn categories(&self) -> &BTreeSet<TagCategory> {
        &self.categories
    }

    pub fn front_image_path(&self) -> &Path {
        &self.front_image_path
    }

    pub fn front_image_uri(&self) -> Cow<'_, str> {
        self.front_image_path.to_string_lossy()
    }

    pub fn back_image_path(&self) -> Option<&Path> {
        self.back_image_path.as_ref().map(|p| p.as_ref())
    }

    pub fn back_image_uri(&self) -> Option<Cow<'_, str>> {
        self.back_image_path.as_ref().map(|p| p.to_string_lossy())
    }
}
