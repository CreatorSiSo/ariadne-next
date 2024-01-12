use crate::style::{Style, Styled};

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
