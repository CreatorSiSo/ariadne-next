use crate::Color;
use std::borrow::Cow;

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

#[derive(Debug, Default, Clone)]
// TODO How and what information should this store? Bitflags?
enum TextStyleFlags {
    #[default]
    None,
    Bold,
    Italic,
    BoldItalic,
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

pub type StyledStr<'a> = Styled<Cow<'a, str>>;

impl<'a> From<&'a str> for StyledStr<'a> {
    fn from(value: &'a str) -> Self {
        Styled::new(value.into(), Style::default())
    }
}

pub trait StyledText<'a> {
    fn parts_vec(self) -> Vec<StyledStr<'a>>;
}

impl<'a> StyledText<'a> for &'a str {
    fn parts_vec(self) -> Vec<StyledStr<'a>> {
        vec![Styled::new(self.into(), Style::default())]
    }
}

impl<'a> StyledText<'a> for String {
    fn parts_vec(self) -> Vec<StyledStr<'a>> {
        vec![Styled::new(self.into(), Style::default())]
    }
}

impl<'a> StyledText<'a> for &[StyledStr<'a>] {
    fn parts_vec(self) -> Vec<StyledStr<'a>> {
        self.into()
    }
}

impl<'a, const L: usize> StyledText<'a> for [StyledStr<'a>; L] {
    fn parts_vec(self) -> Vec<StyledStr<'a>> {
        self.into()
    }
}

impl<'a> StyledText<'a> for Vec<StyledStr<'a>> {
    fn parts_vec(self) -> Vec<StyledStr<'a>> {
        self
    }
}

pub trait StyleExt<'a>: Sized + Into<Cow<'a, str>> {
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
