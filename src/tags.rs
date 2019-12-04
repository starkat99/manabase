use crate::{
    color::{Color, Colors},
    scryfall::{Card, CardFace},
};
use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, HashMap},
    hash::{Hash, Hasher},
    path::Path,
    ptr,
};

lazy_static! {
    static ref LAND_TYPE_REGEX: Regex = { Regex::new(r"\bLand\b").unwrap() };
    static ref ROCK_TYPE_REGEX: Regex = { Regex::new(r"\bArtifact\b").unwrap() };
    static ref DORK_TYPE_REGEX: Regex = { Regex::new(r"\bCreature\b").unwrap() };
    static ref TAG_NAME_STRIP_REGEX: Regex = { Regex::new(r"[^-\w]").unwrap() };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Category {
    Lands,
    Rocks,
    Dorks,
    Ramp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TagKind {
    ColorIdentity,
    ManaPool,
    Cost,
    #[serde(rename = "type")]
    TypeLine,
    Other,
}

#[derive(Debug)]
pub struct TagDb<'a> {
    kind_index: BTreeMap<TagKind, Vec<&'a TagData>>,
}

#[derive(Debug)]
pub struct TagIndex(HashMap<String, TagData>);

#[derive(Debug)]
pub struct TagData {
    name: String,
    conditions: Vec<TagCondition>,
    alt_names: BTreeSet<String>,
    subtags: BTreeSet<String>,
    canonical_name: String,
    kind: TagKind,
    color_identity: Option<Colors>,
}

#[derive(Debug)]
pub struct TagCondition {
    category: Category,
    type_regex: Option<Regex>,
    text_regex: Option<Regex>,
    name_regex: Option<Regex>,
    color_identity_len: Option<usize>,
    color_identity: Option<Vec<Color>>,
    card_face: Option<usize>,
    cmc: Option<f32>,
}

#[derive(Debug)]
pub struct CategoryConfig(HashMap<Category, TagConfigFile>);

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
struct TagConfigFile(HashMap<String, TagConfig>);

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct TagConfig {
    #[serde(default, rename = "type")]
    type_regex: Option<String>,
    #[serde(default, rename = "text")]
    text_regex: Option<String>,
    #[serde(default, rename = "name")]
    name_regex: Option<String>,
    #[serde(default)]
    color_identity_len: Option<usize>,
    #[serde(default)]
    color_identity: Option<Vec<Color>>,
    #[serde(default)]
    card_face: Option<usize>,
    #[serde(default)]
    cmc: Option<f32>,
    #[serde(default)]
    alt_names: Vec<String>,
    #[serde(default)]
    subtags: Vec<String>,
    #[serde(default)]
    kind: TagKind,
}

pub fn load_tags(config_dir: &Path) -> Result<TagIndex, Box<dyn std::error::Error>> {
    let mut config = CategoryConfig(HashMap::new());
    debug!("loading land tags config file");
    config.0.insert(
        Category::Lands,
        toml::from_str(&std::fs::read_to_string(config_dir.join("lands.toml"))?)?,
    );
    debug!("loading rocks tags config file");
    config.0.insert(
        Category::Rocks,
        toml::from_str(&std::fs::read_to_string(config_dir.join("rocks.toml"))?)?,
    );
    debug!("loading dorks tags config file");
    config.0.insert(
        Category::Dorks,
        toml::from_str(&std::fs::read_to_string(config_dir.join("dorks.toml"))?)?,
    );
    debug!("loading ramp tags config file");
    config.0.insert(
        Category::Ramp,
        toml::from_str(&std::fs::read_to_string(config_dir.join("ramp.toml"))?)?,
    );
    debug!("indexing tags");
    Ok(TagIndex::from_config(config)?)
}

impl Category {
    pub fn type_regex(self) -> Option<&'static Regex> {
        match self {
            Category::Lands => Some(&LAND_TYPE_REGEX),
            Category::Rocks => Some(&ROCK_TYPE_REGEX),
            Category::Dorks => Some(&DORK_TYPE_REGEX),
            Category::Ramp => None,
        }
    }

    pub fn tag_name(self) -> &'static str {
        match self {
            Category::Lands => "Land",
            Category::Rocks => "Rock",
            Category::Dorks => "Dork",
            Category::Ramp => "Ramp",
        }
    }

    pub fn base_uri(self) -> &'static str {
        match self {
            Category::Lands => "lands.html",
            Category::Rocks => "rocks.html",
            Category::Dorks => "dorks.html",
            Category::Ramp => "ramp.html",
        }
    }
}

impl std::fmt::Display for Category {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            fmt,
            "{}",
            match self {
                Category::Lands => "Lands",
                Category::Rocks => "Rocks",
                Category::Dorks => "Dorks",
                Category::Ramp => "Ramp",
            }
        )
    }
}

impl TagKind {
    pub fn class(self) -> &'static str {
        match self {
            TagKind::Other => "badge-dark",
            TagKind::ColorIdentity => "badge-warning",
            TagKind::ManaPool => "badge-primary",
            TagKind::TypeLine => "badge-info",
            TagKind::Cost => "badge-secondary",
        }
    }

    pub fn sort_tags(self, tags: &mut Vec<&TagData>) {
        match self {
            // TODO: Fix None sort order
            TagKind::ColorIdentity => tags.sort_unstable_by_key(|tag| &tag.color_identity),
            _ => tags.sort_unstable_by_key(|tag| &tag.name),
        }
    }
}

impl std::fmt::Display for TagKind {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            fmt,
            "{}",
            match self {
                TagKind::ColorIdentity => "Color Identity",
                TagKind::Cost => "Converted Mana Cost",
                TagKind::ManaPool => "Mana",
                TagKind::TypeLine => "Type",
                TagKind::Other => "Other",
            }
        )
    }
}

impl<'a> TagDb<'a> {
    pub fn new(index: &'a TagIndex) -> TagDb<'a> {
        let mut kind_index: BTreeMap<TagKind, Vec<&'a TagData>> = BTreeMap::new();
        for (_, tag) in index.iter() {
            if !kind_index.get(&tag.kind).is_some() {
                kind_index.insert(tag.kind, Vec::new());
            }
            let taglist = kind_index.get_mut(&tag.kind).unwrap();
            taglist.push(&tag);
        }
        for (kind, taglist) in kind_index.iter_mut() {
            kind.sort_tags(taglist);
        }
        TagDb { kind_index }
    }

    pub fn kind_index(&self) -> &BTreeMap<TagKind, Vec<&TagData>> {
        &self.kind_index
    }

    pub fn has_kind_tags_in_category(&self, kind: &TagKind, category: &Category) -> bool {
        self.kind_index[kind]
            .iter()
            .any(|tag| tag.has_category(*category))
    }
}

impl TagIndex {
    fn from_config(config: CategoryConfig) -> Result<Self, regex::Error> {
        let mut tags: HashMap<String, TagData> = HashMap::new();
        for (category, tag_configs) in config.0 {
            for (name, tag_config) in tag_configs.0 {
                if let Some(tag) = tags.get_mut(&name) {
                    tag.merge_config(category, tag_config)?;
                } else {
                    tags.insert(
                        name.clone().into(),
                        TagData::from_config(&name, category, tag_config)?,
                    );
                }
            }
        }
        Ok(TagIndex(tags))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &TagData)> {
        self.0.iter().map(|(s, d)| (s.as_ref(), d))
    }

    pub fn get(&self, tag: &str) -> Option<&TagData> {
        self.0.get(tag)
    }
}

impl TagData {
    fn from_config(
        name: &str,
        category: Category,
        config: TagConfig,
    ) -> Result<TagData, regex::Error> {
        Ok(TagData {
            name: name.to_owned(),
            conditions: vec![TagCondition::from_config(category, &config)?],
            alt_names: config.alt_names.into_iter().collect(),
            subtags: config.subtags.into_iter().collect(),
            canonical_name: TAG_NAME_STRIP_REGEX.replace_all(name, "_").to_string(),
            kind: config.kind,
            color_identity: config.color_identity.clone().map(Colors::from_vec),
        })
    }

    fn merge_config(&mut self, category: Category, config: TagConfig) -> Result<(), regex::Error> {
        self.conditions
            .push(TagCondition::from_config(category, &config)?);
        self.alt_names.extend(config.alt_names);
        self.subtags.extend(config.subtags);
        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item = &TagCondition> {
        self.conditions.iter()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> TagKind {
        self.kind
    }

    pub fn color_identity_symbols(&self) -> Cow<'static, str> {
        if let Some(color_identity) = &self.color_identity {
            color_identity.mana_symbols()
        } else {
            Cow::Borrowed("")
        }
    }

    pub fn canonical_name(&self) -> &str {
        &self.canonical_name
    }

    pub fn has_category(&self, category: Category) -> bool {
        self.conditions.iter().any(|c| c.category == category)
    }
}

trait MatchItem {
    fn type_line(&self) -> Option<&str>;
    fn name(&self) -> &str;
    fn text(&self) -> Option<&str>;
}

impl TagCondition {
    fn from_config(category: Category, config: &TagConfig) -> Result<TagCondition, regex::Error> {
        Ok(TagCondition {
            category,
            type_regex: config
                .type_regex
                .as_ref()
                .map(|s| {
                    let mut builder = RegexBuilder::new(&s);
                    builder.dot_matches_new_line(true);
                    builder.build()
                })
                .transpose()?,
            text_regex: config
                .text_regex
                .as_ref()
                .map(|s| {
                    let mut builder = RegexBuilder::new(&s);
                    builder.dot_matches_new_line(true);
                    builder.build()
                })
                .transpose()?,
            name_regex: config
                .name_regex
                .as_ref()
                .map(|s| {
                    let mut builder = RegexBuilder::new(&s);
                    builder.dot_matches_new_line(true);
                    builder.build()
                })
                .transpose()?,
            color_identity: config.color_identity.clone(),
            color_identity_len: config.color_identity_len,
            card_face: config.card_face,
            cmc: config.cmc,
        })
    }

    pub fn category(&self) -> Category {
        self.category
    }

    pub fn is_match(&self, card: &Card) -> bool {
        if let Some(cmc) = self.cmc {
            if card.cmc != cmc {
                return false;
            }
        }

        if let Some(color_identity_len) = self.color_identity_len {
            if card.color_identity.len() != color_identity_len {
                return false;
            }
        }

        if let Some(color_identity) = &self.color_identity {
            if card.color_identity.len() != color_identity.len() {
                return false;
            }
            for color in color_identity {
                if !card.color_identity.contains(color) {
                    return false;
                }
            }
        }

        // First, check for a specified face and only use that face
        if let Some(face) = self
            .card_face
            .and_then(|i| card.card_faces.as_ref().and_then(|v| v.get(i)))
        {
            if !self.is_item_match(face) {
                return false;
            }
        } else {
            if !self.is_item_match(card) {
                return false;
            }
        }

        true
    }

    fn is_item_match<T: MatchItem>(&self, card: &T) -> bool {
        // Check category first
        if !card
            .type_line()
            .filter(|s| {
                self.category
                    .type_regex()
                    .filter(|r| !r.is_match(s))
                    .is_none()
            })
            .is_some()
        {
            return false;
        }

        if let Some(type_regex) = &self.type_regex {
            if !card
                .type_line()
                .filter(|s| type_regex.is_match(s))
                .is_some()
            {
                return false;
            }
        }

        if let Some(name_regex) = &self.name_regex {
            if !name_regex.is_match(card.name()) {
                return false;
            }
        }

        if let Some(text_regex) = &self.text_regex {
            if !card.text().filter(|s| text_regex.is_match(s)).is_some() {
                return false;
            }
        }

        true
    }
}

impl<'a> MatchItem for Card<'a> {
    fn type_line(&self) -> Option<&str> {
        self.type_line.as_ref().map(|s| s.as_ref())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn text(&self) -> Option<&str> {
        self.oracle_text.as_ref().map(|s| s.as_ref())
    }
}

impl<'a> MatchItem for CardFace<'a> {
    fn type_line(&self) -> Option<&str> {
        self.type_line.as_ref().map(|s| s.as_ref())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn text(&self) -> Option<&str> {
        self.oracle_text.as_ref().map(|s| s.as_ref())
    }
}

impl PartialEq<&TagData> for &TagData {
    fn eq(&self, other: &&TagData) -> bool {
        ptr::eq(*self as *const TagData, *other as *const TagData)
    }
}

impl Eq for &TagData {}

impl Hash for &TagData {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        ptr::hash(self as *const &TagData, hasher)
    }
}

impl Default for TagKind {
    fn default() -> Self {
        TagKind::Other
    }
}

impl std::fmt::Display for TagData {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.kind {
            TagKind::ColorIdentity => write!(
                fmt,
                "Color: {} {}",
                &self.color_identity_symbols(),
                &self.name
            ),
            TagKind::ManaPool => write!(fmt, "Mana: {}", &self.name),
            TagKind::Cost => write!(fmt, "Cost: {}", &self.name),
            _ => write!(fmt, "{}", &self.name),
        }
    }
}
