use crate::Backend;

pub struct VStack {
    pub elements: Vec<Element>,
}

impl VStack {
    pub fn new() -> Self {
        Self { elements: vec![] }
    }

    pub fn write<B: Backend>(&self, backend: &mut B) -> Result<(), B::Error> {
        backend.write(self)
    }
}

pub enum Element {
    // TODO VStack should go in here
    HStack(Vec<Element>),
    Container {
        content: Vec<Inline>,
        width: Option<usize>,
        height: Option<usize>,
        style: TextStyle,
    },
    Inline(Inline),
}

pub trait ToElement {
    fn into_element(self) -> Element;
}

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

// TODO Should this be implemented for everything that implenents Display?
// TODO How to improve error messages? ie. for custom Level
impl<S: Into<String>> ToElement for S {
    fn into_element(self) -> Element {
        Element::Inline(Inline::new(self.into()))
    }
}

#[derive(Default)]
pub struct TextStyle {
    /// Color of the text
    fg_color: Option<Color>,
    /// Color of the background
    bg_color: Option<Color>,
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

#[derive(Default)]
// TODO How and what information should this store? Bitflags?
enum TextStyleFlags {
    #[default]
    None,
    Bold,
    Italic,
    BoldItalic,
}

// TODO There should probably be some way of using presets (like red, green, yellow, ...)
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}
