use crate::Color;

#[derive(Debug, Clone)]
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

pub trait ElementLayout {
    fn element_layout(self) -> Element;
}

impl<T: InlineLayout> ElementLayout for T {
    fn element_layout(self) -> Element {
        Element::Inline(self.inline_layout())
    }
}

impl ElementLayout for Inline {
    fn element_layout(self) -> Element {
        Element::Inline(self)
    }
}

#[derive(Debug, Clone)]
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
}

pub trait InlineLayout {
    fn inline_layout(self) -> Inline;
}

impl<T: std::fmt::Display> InlineLayout for T {
    fn inline_layout(self) -> Inline {
        Inline::new(self.to_string())
    }
}

pub trait Styled<T> {
    fn with_style(self, style: TextStyle) -> T;
}

impl Styled<Inline> for Inline {
    fn with_style(mut self, style: TextStyle) -> Inline {
        self.style = style;
        self
    }
}

impl<T: InlineLayout> Styled<Inline> for T {
    fn with_style(self, style: TextStyle) -> Inline {
        let mut inline = self.inline_layout();
        inline.style = style;
        inline
    }
}

#[derive(Debug, Default, Clone)]
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

    pub fn with_fg(mut self, color: Color) -> Self {
        self.fg_color = Some(color);
        self
    }

    pub fn with_bg(mut self, color: Color) -> Self {
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

pub mod shortcuts {
    use super::{Element, InlineLayout};

    pub fn inline(text: impl InlineLayout) -> Element {
        Element::Inline(text.inline_layout())
    }
}
