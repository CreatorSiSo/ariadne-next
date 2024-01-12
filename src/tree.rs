use std::borrow::Cow;

use crate::{Color, StyledStr};

// TODO Replace with From<T>?
pub trait Layout {
    fn layout(self) -> Element;
}

impl<T: std::fmt::Display> Layout for T {
    fn layout(self) -> Element {
        Element::Inline {
            text: self.to_string(),
        }
    }
}

// TODO Explain all kinds of elements
#[derive(Debug, Clone)]
pub enum Element {
    VStack {
        children: Vec<Styled<Element>>,
    },
    HStack {
        children: Vec<Styled<Element>>,
    },
    // Example of wrapping:
    // --------------------------------------------
    // | Inline | VStack | Box with width | Inlin |
    // |--------| VStack | an some wrappi |--------
    // |        | VStack | ng content     |       |
    // |        | VStack |-----------------       |
    // |        | VStack |                        |
    // |--------------------------                |
    // | e that doesnt quiet fit |                |
    // |--------------------------------------    |
    // | Elements that wont be broken apart: |    |
    // | - VStacks (like this one)           |    |
    // | - HStacks                           |    |
    // | - Boxes with width                  |    |
    // --------------------------------------------
    Box {
        children: Vec<Styled<Element>>,
        /// Unicode width of the content
        /// - Some: Height is adjusted to fit the entire content
        /// - None: Height is always that of the tallest child
        width: Option<usize>,
    },
    Inline {
        text: String,
    },
}

impl Element {
    pub fn vstack(elements: impl IntoIterator<Item = Styled<Element>>) -> Self {
        Self::VStack {
            children: elements.into_iter().collect(),
        }
    }

    pub fn hstack(elements: impl IntoIterator<Item = Styled<Element>>) -> Self {
        Self::HStack {
            children: elements.into_iter().collect(),
        }
    }

    pub fn box_(elements: impl IntoIterator<Item = Styled<Element>>, width: Option<usize>) -> Self {
        Self::Box {
            children: elements.into_iter().collect(),
            width,
        }
    }

    pub fn inline(text: impl ToString) -> Self {
        Self::Inline {
            text: text.to_string(),
        }
    }

    pub fn styled(self, style: Style) -> Styled<Self> {
        Styled::new(self, style)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Style {
    /// Color of the text
    pub fg_color: Option<Color>,
    /// Color of the background
    pub bg_color: Option<Color>,
    /// Additional formatting data
    flags: TextStyleFlags,
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fg(mut self, color: Color) -> Self {
        self.fg_color = Some(color);
        self
    }

    pub fn bg(mut self, color: Color) -> Self {
        self.bg_color = Some(color);
        self
    }

    pub fn bold(mut self) -> Self {
        self.flags = match self.flags {
            TextStyleFlags::None | TextStyleFlags::Bold => TextStyleFlags::Bold,
            TextStyleFlags::Italic | TextStyleFlags::BoldItalic => TextStyleFlags::BoldItalic,
        };
        self
    }

    pub fn italic(mut self) -> Self {
        self.flags = match self.flags {
            TextStyleFlags::None | TextStyleFlags::Italic => TextStyleFlags::Italic,
            TextStyleFlags::Bold | TextStyleFlags::BoldItalic => TextStyleFlags::BoldItalic,
        };
        self
    }

    pub fn is_bold(&self) -> bool {
        matches!(
            self.flags,
            TextStyleFlags::Bold | TextStyleFlags::BoldItalic
        )
    }

    pub fn is_italic(&self) -> bool {
        matches!(
            self.flags,
            TextStyleFlags::Italic | TextStyleFlags::BoldItalic
        )
    }

    // TODO add set_bold and set_italic
}

#[derive(Debug)]
pub struct Styled<T: std::fmt::Debug> {
    inner: T,
    style: Style,
}

impl<T: std::fmt::Debug> Styled<T> {
    pub fn new(inner: T, style: Style) -> Self {
        Self { inner, style }
    }

    pub fn default(inner: T) -> Self {
        Self {
            inner,
            style: Style::default(),
        }
    }

    pub fn map<O: std::fmt::Debug>(self, f: impl Fn(T) -> O) -> Styled<O> {
        Styled::new(f(self.inner), self.style)
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn style(&self) -> &Style {
        &self.style
    }
}

impl<T: Clone + std::fmt::Debug> Clone for Styled<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            style: self.style.clone(),
        }
    }
}

pub trait StyleExt<'a>: std::fmt::Debug + Sized + Into<Cow<'a, str>> {
    fn fg(self, color: Color) -> StyledStr<'a> {
        Styled::new(self.into(), Style::new().fg(color))
    }

    fn bg(self, color: Color) -> StyledStr<'a> {
        Styled::new(self.into(), Style::new().bg(color))
    }

    fn bold(self) -> StyledStr<'a> {
        Styled::new(self.into(), Style::new().bold())
    }

    fn italic(self) -> StyledStr<'a> {
        Styled::new(self.into(), Style::new().italic())
    }
}

impl<'a> StyleExt<'a> for &'a str {}
impl StyleExt<'_> for String {}

#[derive(Debug, Default, Clone)]
// TODO How and what information should this store? Bitflags?
enum TextStyleFlags {
    #[default]
    None,
    Bold,
    Italic,
    BoldItalic,
}

pub enum BasicColor {
    Unset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}
