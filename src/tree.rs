use crate::Color;

// TODO Replace with From<T>?
pub trait Layout {
    fn layout(self) -> Element;
}

impl<T: std::fmt::Display> Layout for T {
    fn layout(self) -> Element {
        Element::Inline {
            text: self.to_string(),
            style: Style::default(),
        }
    }
}

// TODO Explain all kinds of elements
#[derive(Debug, Clone)]
pub enum Element {
    VStack {
        children: Vec<Element>,
        style: Style,
    },
    HStack {
        children: Vec<Element>,
        style: Style,
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
        children: Vec<Element>,
        /// Unicode width of the content
        /// - Some: Height is adjusted to fit the entire content
        /// - None: Height is always that of the tallest child
        width: Option<usize>,
        style: Style,
    },
    Inline {
        text: String,
        style: Style,
    },
}

impl Element {
    pub fn vstack(elements: impl IntoIterator<Item = Element>) -> Self {
        Self::VStack {
            children: elements.into_iter().collect(),
            style: Style::default(),
        }
    }

    pub fn hstack(elements: impl IntoIterator<Item = Element>) -> Self {
        Self::HStack {
            children: elements.into_iter().collect(),
            style: Style::default(),
        }
    }

    pub fn box_(elements: impl IntoIterator<Item = Element>, width: Option<usize>) -> Self {
        Self::Box {
            children: elements.into_iter().collect(),
            width,
            style: Style::default(),
        }
    }

    pub fn inline(text: impl ToString) -> Self {
        Self::Inline {
            text: text.to_string(),
            style: Style::default(),
        }
    }

    pub fn style(&self) -> &Style {
        match self {
            Element::VStack { style, .. } => style,
            Element::HStack { style, .. } => style,
            Element::Box { style, .. } => style,
            Element::Inline { style, .. } => style,
        }
    }
}

impl Styled<Element> for Element {
    fn with_style(self, style: Style) -> Element {
        match self {
            Element::VStack {
                children: elements, ..
            } => Element::VStack {
                children: elements,
                style,
            },
            Element::HStack {
                children: elements, ..
            } => Element::HStack {
                children: elements,
                style,
            },
            Element::Box {
                children: elements,
                width,
                ..
            } => Element::Box {
                children: elements,
                width,
                style,
            },
            Element::Inline { text, .. } => Element::Inline { text, style },
        }
    }
}

impl<T: Layout> Styled<Element> for T {
    fn with_style(self, style: Style) -> Element {
        self.layout().with_style(style)
    }
}

pub trait Styled<T> {
    fn with_style(self, style: Style) -> T;
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
