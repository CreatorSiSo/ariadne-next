use crate::{Backend, Color};

#[derive(Debug)]
pub enum Element {
    VStack(Vec<Element>),
    HStack(Vec<Element>),
    Box {
        content: Vec<Inline>,
        width: Option<usize>,
        style: TextStyle,
    },
    Inline(Inline),
}

impl Element {
    pub fn write<B: Backend>(&self, backend: &mut B) -> Result<(), B::Error> {
        backend.write(self)
    }
}

pub trait IntoElement {
    fn into_element(self) -> Element;
}

// TODO Should this be implemented for everything that implenents Display?
// TODO How to improve error messages?
impl<S: Into<String>> IntoElement for S {
    fn into_element(self) -> Element {
        Element::Inline(Inline::new(self.into()))
    }
}

#[derive(Debug)]
pub struct Inline {
    pub text: String,
    pub style: TextStyle,
}

impl Inline {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: TextStyle::default(),
        }
    }

    pub fn with_style(mut self, style: TextStyle) -> Self {
        self.style = style;
        self
    }
}

#[derive(Debug, Default)]
pub struct TextStyle {
    /// Color of the text
    pub fg_color: Option<Color>,
    /// Color of the background
    pub bg_color: Option<Color>,
    /// Additional formatting data
    flags: TextStyleFlags,
}

impl TextStyle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_fg_color(mut self, color: Color) -> Self {
        self.fg_color = Some(color);
        self
    }

    pub fn with_bg_color(mut self, color: Color) -> Self {
        self.bg_color = Some(color);
        self
    }

    pub fn with_bold(mut self) -> Self {
        self.flags = match self.flags {
            TextStyleFlags::None | TextStyleFlags::Bold => TextStyleFlags::Bold,
            TextStyleFlags::Italic | TextStyleFlags::BoldItalic => TextStyleFlags::BoldItalic,
        };
        self
    }

    pub fn with_italic(mut self) -> Self {
        self.flags = match self.flags {
            TextStyleFlags::None | TextStyleFlags::Italic => TextStyleFlags::Italic,
            TextStyleFlags::Bold | TextStyleFlags::BoldItalic => TextStyleFlags::BoldItalic,
        };
        self
    }

    // TODO add set_bold and set_italic
}

#[derive(Debug, Default)]
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
