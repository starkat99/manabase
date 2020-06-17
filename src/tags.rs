use crate::{
    color::{Color, Colors},
    scryfall::{Card, Format, Legality},
};
use itertools::free::join;
use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    hash::{Hash, Hasher},
    ops::Deref,
    path::Path,
    ptr,
};

lazy_static! {
    static ref TAG_NAME_STRIP_REGEX: Regex = Regex::new(r"[^-\w]").unwrap();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TagKind {
    ColorIdentity,
    ManaPool,
    Cost,
    #[serde(rename = "type")]
    TypeLine,
    Format,
    Other,
}

#[derive(Debug)]
pub struct TagDb<'a> {
    kind_index: BTreeMap<TagKind, Vec<TagRef<'a>>>,
}

#[derive(Debug)]
pub struct TagIndex(HashMap<String, TagData>);

#[derive(Debug)]
pub struct TagData {
    name: String,
    alt_names: BTreeSet<String>,
    description: Option<String>,
    subtags: BTreeSet<String>,
    canonical_name: String,
    kind: TagKind,
    cmc: Option<f32>,
    type_regex: Option<Regex>,
    color_identity: Option<Colors>,
    mana: Option<Colors>,
    format: Option<(Format, Legality)>,
}

#[derive(Debug)]
pub struct TagRef<'a>(&'a TagData);

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
struct TagConfigFile(HashMap<String, TagConfig>);

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct TagConfig {
    #[serde(default, rename = "type")]
    type_regex: Option<String>,
    #[serde(default)]
    color_identity: Option<Vec<Color>>,
    #[serde(default)]
    mana: Option<Vec<Color>>,
    #[serde(default)]
    cmc: Option<f32>,
    #[serde(default)]
    alt_names: Vec<String>,
    #[serde(default)]
    subtags: Vec<String>,
    #[serde(default)]
    kind: TagKind,
    #[serde(default)]
    format: Option<Format>,
    #[serde(default)]
    legality: Option<Legality>,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CardTags(HashMap<String, Vec<String>>);

impl TagKind {
    pub fn class(self) -> &'static str {
        match self {
            TagKind::Other => "badge-dark",
            TagKind::ColorIdentity => "badge-warning",
            TagKind::ManaPool => "badge-success",
            TagKind::TypeLine => "badge-info",
            TagKind::Cost => "badge-secondary",
            TagKind::Format => "badge-danger",
        }
    }

    pub fn sort_tags(self, tags: &mut Vec<TagRef<'_>>) {
        match self {
            TagKind::ColorIdentity => tags.sort_unstable_by_key(|tag| tag.color_identity),
            TagKind::ManaPool => tags.sort_unstable_by_key(|tag| tag.mana),
            TagKind::Cost => tags.sort_unstable_by_key(|tag| tag.cmc.map(|m| m as i32)),
            _ => tags.sort_unstable_by_key(|tag| tag.name.clone()),
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
                TagKind::Format => "Format Legality",
                TagKind::Other => "Other",
            }
        )
    }
}

impl<'a> TagDb<'a> {
    pub fn new(index: &'a TagIndex) -> TagDb<'a> {
        let mut kind_index: BTreeMap<TagKind, Vec<TagRef<'a>>> = BTreeMap::new();
        for (_, tag) in index.iter() {
            if !kind_index.get(&tag.kind).is_some() {
                kind_index.insert(tag.kind, Vec::new());
            }
            let taglist = kind_index.get_mut(&tag.kind).unwrap();
            taglist.push(tag);
        }
        for (kind, taglist) in kind_index.iter_mut() {
            kind.sort_tags(taglist);
        }
        TagDb { kind_index }
    }

    pub fn kind_index(&self) -> &BTreeMap<TagKind, Vec<TagRef<'_>>> {
        &self.kind_index
    }
}

impl TagIndex {
    pub fn load(config_file: &Path) -> anyhow::Result<TagIndex> {
        debug!("loading tags config file");
        let config = toml::from_str(&std::fs::read_to_string(config_file)?)?;
        debug!("indexing tags");
        Ok(TagIndex::from_config(config)?)
    }

    fn from_config(config: TagConfigFile) -> Result<Self, regex::Error> {
        let mut tags: HashMap<_, _> = HashMap::new();
        for (name, tag_config) in config.0 {
            tags.insert(
                name.clone().into(),
                TagData::from_config(&name, tag_config)?,
            );
        }

        Ok(TagIndex(tags))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, TagRef<'_>)> {
        self.0.iter().map(|(s, d)| (s.as_ref(), TagRef::new(d)))
    }

    pub fn get(&self, tag: &str) -> Option<TagRef<'_>> {
        self.0.get(tag).map(TagRef::new)
    }

    pub fn merge_tags(&mut self, card_tags: &CardTags) {
        for tag in card_tags.tags() {
            if !self.0.contains_key(tag) {
                self.0.insert(tag.to_owned(), TagData::new(tag));
            }
        }
    }
}

impl TagData {
    fn new(name: &str) -> TagData {
        TagData {
            name: name.to_owned(),
            alt_names: Default::default(),
            subtags: Default::default(),
            canonical_name: TAG_NAME_STRIP_REGEX.replace_all(name, "_").to_string(),
            kind: TagKind::Other,
            color_identity: None,
            cmc: None,
            type_regex: None,
            mana: None,
            format: None,
            description: None,
        }
    }

    fn from_config(name: &str, config: TagConfig) -> Result<TagData, regex::Error> {
        Ok(TagData {
            name: name.to_owned(),
            alt_names: config.alt_names.into_iter().collect(),
            subtags: config.subtags.into_iter().collect(),
            canonical_name: TAG_NAME_STRIP_REGEX.replace_all(name, "_").to_string(),
            kind: config.kind,
            color_identity: config.color_identity.clone().map(Colors::from_vec),
            cmc: config.cmc,
            type_regex: config
                .type_regex
                .as_ref()
                .map(|s| {
                    let mut builder = RegexBuilder::new(&s);
                    builder.dot_matches_new_line(true);
                    builder.build()
                })
                .transpose()?,
            mana: config.mana.clone().map(Colors::from_vec),
            format: match (config.format, config.legality) {
                (Some(f), Some(l)) => Some((f, l)),
                _ => None,
            },
            description: config.description,
        })
    }

    pub fn name(&self) -> Cow<'_, str> {
        match self.kind {
            TagKind::Cost => Cow::Owned(format!("CMC: {}", self.cmc.unwrap_or_default() as i32)),
            TagKind::Format => Cow::Owned(format!(
                "{} in {}",
                self.format.unwrap().1,
                self.format.unwrap().0
            )),
            _ => Cow::Borrowed(&self.name),
        }
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

    pub fn mana_symbols(&self) -> Cow<'static, str> {
        if let Some(mana) = &self.mana {
            mana.mana_symbols()
        } else {
            Cow::Borrowed("")
        }
    }

    pub fn cmc_symbol(&self) -> Cow<'static, str> {
        if let Some(cmc) = &self.cmc {
            Cow::Owned(format!("<span class=\"mana s{}\"></span>", *cmc as i32))
        } else {
            Cow::Borrowed("")
        }
    }

    pub fn canonical_name(&self) -> &str {
        &self.canonical_name
    }

    pub fn subtags(&self) -> &BTreeSet<String> {
        &self.subtags
    }

    pub fn has_alt_names(&self) -> bool {
        !self.alt_names.is_empty()
    }

    pub fn alt_names_string(&self) -> String {
        join(self.alt_names.iter(), ", ")
    }

    pub fn is_match(&self, card: &Card, type_line: &str) -> bool {
        if let Some(cmc) = self.cmc {
            if card.cmc == cmc && (!type_line.contains("Land") || cmc > 0.0) {
                return true;
            }
        }

        if let Some(color_identity) = &self.color_identity {
            if &Colors::from_vec(card.color_identity.clone()) == color_identity {
                return true;
            }
        }

        if let Some(type_regex) = &self.type_regex {
            if type_regex.is_match(type_line.as_ref()) {
                return true;
            }
        }

        if let Some((format, legality)) = &self.format {
            if card
                .legalities
                .as_ref()
                .and_then(|legalities| legalities.get(format))
                .filter(|l| l == &legality)
                .is_some()
            {
                return true;
            }
        }

        false
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
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
                "Color Identity: {} {}",
                self.color_identity_symbols(),
                &self.name
            ),
            TagKind::ManaPool => write!(
                fmt,
                "Mana: {} {}",
                self.mana_symbols(),
                &self.name.replace(" Mana", "")
            ),
            TagKind::Cost => write!(fmt, "CMC: {}", self.cmc_symbol()),
            TagKind::Format => match self.format.unwrap() {
                (f, Legality::Legal) => write!(fmt, "{}", f),
                (f, l) => write!(fmt, "{}: {}", f, l),
            },
            _ => write!(fmt, "{}", &self.name),
        }
    }
}

impl<'a> TagRef<'a> {
    pub fn new(tag: &'a TagData) -> TagRef<'a> {
        TagRef(tag)
    }
}

impl<'a> Clone for TagRef<'a> {
    fn clone(&self) -> TagRef<'a> {
        TagRef(self.0)
    }
}

impl<'a> Copy for TagRef<'a> {}

impl<'a> PartialEq for TagRef<'a> {
    fn eq(&self, other: &TagRef) -> bool {
        ptr::eq(self.0, other.0)
    }
}

impl<'a> Eq for TagRef<'a> {}

impl<'a> Hash for TagRef<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ptr::hash(self.0, state)
    }
}

impl<'a> AsRef<TagData> for TagRef<'a> {
    fn as_ref(&self) -> &TagData {
        self.0
    }
}

impl<'a> Deref for TagRef<'a> {
    type Target = TagData;
    fn deref(&self) -> &TagData {
        self.0
    }
}

impl<'a> std::fmt::Display for TagRef<'a> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(fmt)
    }
}

impl CardTags {
    pub fn load(config_file: &Path) -> anyhow::Result<CardTags> {
        debug!("loading card tag list");
        Ok(toml::from_str(&std::fs::read_to_string(config_file)?)?)
    }

    pub fn cards(&self) -> impl Iterator<Item = &str> {
        self.0.keys().map(|s| s.as_ref())
    }

    fn tags(&self) -> HashSet<&str> {
        self.0.values().flatten().map(|s| s.as_ref()).collect()
    }

    pub fn get_tags(&self, name: &str) -> Option<&Vec<String>> {
        self.0.get(name)
    }
}
