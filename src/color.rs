use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    fmt::{Display, Formatter},
};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Color {
    #[serde(rename = "W")]
    White,
    #[serde(rename = "U")]
    Blue,
    #[serde(rename = "B")]
    Black,
    #[serde(rename = "R")]
    Red,
    #[serde(rename = "G")]
    Green,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum DualColor {
    Azorius,
    Dimir,
    Rakdos,
    Gruul,
    Selesnya,
    Orzhov,
    Boros,
    Izzet,
    Simic,
    Golgari,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum TriColor {
    Bant,
    Esper,
    Grixis,
    Jund,
    Naya,
    Abzan,
    Jeskai,
    Sultai,
    Mardu,
    Temur,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum QuadColor {
    Artifice,
    Chaos,
    Aggression,
    Altruism,
    Growth,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum Colors {
    Colorless,
    Mono(Color),
    Dual(DualColor),
    Tri(TriColor),
    Quad(QuadColor),
    Domain,
}

impl Color {
    pub fn mana_symbol(self) -> &'static str {
        match self {
            Color::White => "<span class=\"mana sw\"></span>",
            Color::Blue => "<span class=\"mana su\"></span>",
            Color::Black => "<span class=\"mana sb\"></span>",
            Color::Red => "<span class=\"mana sr\"></span>",
            Color::Green => "<span class=\"mana sg\"></span>",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Color::White => "White",
            Color::Blue => "Blue",
            Color::Black => "Black",
            Color::Red => "Red",
            Color::Green => "Green",
        }
    }
}

impl DualColor {
    pub fn name(self) -> &'static str {
        use DualColor::*;
        match self {
            Azorius => "Azorius",
            Dimir => "Dimir",
            Rakdos => "Rakdos",
            Gruul => "Gruul",
            Selesnya => "Selesnya",
            Orzhov => "Orzhov",
            Boros => "Boros",
            Izzet => "Izzet",
            Simic => "Simic",
            Golgari => "Golgari",
        }
    }
}

impl TriColor {
    pub fn name(self) -> &'static str {
        use TriColor::*;
        match self {
            Bant => "Bant",
            Esper => "Esper",
            Grixis => "Grixis",
            Jund => "Jund",
            Naya => "Naya",
            Abzan => "Abzan",
            Jeskai => "Jeskai",
            Sultai => "Sultai",
            Mardu => "Mardu",
            Temur => "Temur",
        }
    }
}

impl QuadColor {
    pub fn name(self) -> &'static str {
        use QuadColor::*;
        match self {
            Artifice => "Artifice",
            Chaos => "Chaos",
            Aggression => "Aggression",
            Altruism => "Altruism",
            Growth => "Growth",
        }
    }
}

impl Colors {
    pub fn from_vec(mut colors: Vec<Color>) -> Self {
        use Color::*;
        use Colors::*;
        use DualColor::*;
        use QuadColor::*;
        use TriColor::*;
        colors.sort_unstable();
        match colors[..] {
            [] => Colorless,
            [c] => Mono(c),
            [White, Blue] => Dual(Azorius),
            [White, Green] => Dual(Selesnya),
            [White, Black] => Dual(Orzhov),
            [White, Red] => Dual(Boros),
            [Blue, Black] => Dual(Dimir),
            [Blue, Red] => Dual(Izzet),
            [Blue, Green] => Dual(Simic),
            [Black, Red] => Dual(Rakdos),
            [Black, Green] => Dual(Golgari),
            [Red, Green] => Dual(Gruul),
            [White, Blue, Black] => Tri(Esper),
            [White, Blue, Green] => Tri(Bant),
            [White, Blue, Red] => Tri(Jeskai),
            [White, Black, Red] => Tri(Mardu),
            [White, Black, Green] => Tri(Abzan),
            [White, Red, Green] => Tri(Naya),
            [Blue, Black, Red] => Tri(Grixis),
            [Blue, Black, Green] => Tri(Sultai),
            [Blue, Red, Green] => Tri(Temur),
            [Black, Red, Green] => Tri(Jund),
            [White, Blue, Black, Red] => Quad(Artifice),
            [White, Blue, Black, Green] => Quad(Growth),
            [White, Blue, Red, Green] => Quad(Altruism),
            [White, Black, Red, Green] => Quad(Aggression),
            [Blue, Black, Red, Green] => Quad(Chaos),
            [White, Blue, Black, Red, Green] => Domain,
            _ => panic!("unknown color configuration"),
        }
    }

    pub fn into_vec(self) -> Vec<Color> {
        use Color::*;
        use Colors::*;
        use DualColor::*;
        use QuadColor::*;
        use TriColor::*;
        match self {
            Colorless => Vec::new(),
            Mono(c) => [c][..].into(),
            Dual(Azorius) => [White, Blue][..].into(),
            Dual(Dimir) => [Blue, Black][..].into(),
            Dual(Rakdos) => [Black, Red][..].into(),
            Dual(Gruul) => [Red, Green][..].into(),
            Dual(Selesnya) => [Green, White][..].into(),
            Dual(Orzhov) => [White, Black][..].into(),
            Dual(Izzet) => [Blue, Red][..].into(),
            Dual(Golgari) => [Black, Green][..].into(),
            Dual(Boros) => [Red, White][..].into(),
            Dual(Simic) => [Green, Blue][..].into(),
            Tri(Bant) => [Green, White, Blue][..].into(),
            Tri(Esper) => [White, Blue, Black][..].into(),
            Tri(Grixis) => [Blue, Black, Red][..].into(),
            Tri(Jund) => [Black, Red, Green][..].into(),
            Tri(Naya) => [Red, Green, White][..].into(),
            Tri(Abzan) => [White, Black, Green][..].into(),
            Tri(Jeskai) => [Blue, Red, White][..].into(),
            Tri(Sultai) => [Black, Green, Blue][..].into(),
            Tri(Mardu) => [Red, White, Black][..].into(),
            Tri(Temur) => [Green, Blue, Red][..].into(),
            Quad(Artifice) => [White, Blue, Black, Red][..].into(),
            Quad(Chaos) => [Blue, Black, Red, Green][..].into(),
            Quad(Aggression) => [Black, Red, Green, White][..].into(),
            Quad(Altruism) => [Red, Green, White, Blue][..].into(),
            Quad(Growth) => [Green, White, Blue, Black][..].into(),
            Domain => [White, Blue, Black, Red, Green][..].into(),
        }
    }

    pub fn mana_symbols(self) -> Cow<'static, str> {
        match self {
            Colors::Colorless => Cow::Borrowed("<span class=\"mana sc\"></span>"),
            value => Cow::Owned(
                value
                    .into_vec()
                    .iter()
                    .map(|c| c.mana_symbol())
                    .collect::<Vec<_>>()
                    .join(""),
            ),
        }
    }

    pub fn name(self) -> &'static str {
        use Colors::*;
        match self {
            Colorless => "Colorless",
            Mono(c) => c.name(),
            Dual(c) => c.name(),
            Tri(c) => c.name(),
            Quad(c) => c.name(),
            Domain => "Domain",
        }
    }
}

impl Display for Color {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{} {}", self.mana_symbol(), self.name())
    }
}

impl Display for Colors {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{} {}", self.mana_symbols(), self.name())
    }
}
