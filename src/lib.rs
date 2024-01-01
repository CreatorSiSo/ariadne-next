use core::fmt;

mod plaintext;
pub use plaintext::PlainText;

pub type Span = std::ops::Range<usize>;

#[must_use]
pub struct Report<Level> {
    level: Level,
    code: Option<String>,
    message: String,
    /// Annotated section of source code
    view: Option<SourceView<Level>>,
    /// Help or note messages
    comments: Vec<(Level, String)>,
}

impl<Level> Report<Level> {
    pub fn new(level: Level) -> Self {
        Self {
            level,
            code: None,
            message: "".into(),
            view: None,
            comments: vec![],
        }
    }

    pub fn with_view(mut self, view: SourceView<Level>) -> Self {
        self.view = Some(view);
        self
    }

    pub fn set_view(&mut self, view: SourceView<Level>) {
        self.view = Some(view);
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn set_message(&mut self, message: impl Into<String>) {
        self.message = message.into();
    }

    pub fn with_code(mut self, code: impl fmt::Display) -> Self {
        self.code = Some(code.to_string());
        self
    }

    pub fn set_code(&mut self, code: impl Into<String>) {
        self.code = Some(code.into());
    }

    // TODO Comments

    pub fn finish(self) -> Sequence {
        // TODO Generate Sequence of Elements
        Sequence { elements: vec![] }
    }
}

/// Annotated section of source code
pub struct SourceView<Level> {
    // TODO Should we even store source here?
    source: &'static str,
    labels: Vec<Label<Level>>,
}

impl<Level> SourceView<Level> {
    pub fn new(source: &'static str) -> Self {
        Self {
            source,
            labels: vec![],
        }
    }

    pub fn with_label(mut self, label: Label<Level>) -> Self {
        self.labels.push(label);
        self
    }

    pub fn with_labels(mut self, labels: impl IntoIterator<Item = Label<Level>>) -> Self {
        self.labels.extend(labels);
        self
    }

    pub fn add_label(&mut self, label: Label<Level>) {
        self.labels.push(label);
    }

    pub fn add_labels(&mut self, labels: impl IntoIterator<Item = Label<Level>>) {
        self.labels.extend(labels);
    }
}

pub struct Label<Level> {
    level: Level,
    span: Span,
    message: String,
}

impl<Level> Label<Level> {
    pub fn new(level: Level, span: Span) -> Self {
        Self {
            level,
            span,
            message: "".into(),
        }
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn set_message(&mut self, message: impl Into<String>) {
        self.message = message.into();
    }
}

pub struct Sequence {
    elements: Vec<Element>,
}

impl Sequence {
    pub fn write<B: Backend>(&self, backend: &mut B) -> Result<(), B::Error> {
        backend.write(self)
    }
}

pub enum Element {}

pub trait Backend {
    // type Output;
    type Error;

    fn write(&mut self, sequence: &Sequence) -> Result<(), Self::Error>;

    // fn render(sequence: &Sequence) -> Result<Self::Output, Self::Error>;
}
