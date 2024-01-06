use std::{collections::HashMap, fmt::Debug, path::PathBuf};
use std::{fmt, fs, io};

pub mod tree;
use tree::{Element, ElementLayout};

mod backends;
pub use backends::{Ansi, PlainText};

pub type Color = yansi::Color;
pub type Span = std::ops::Range<usize>;

// TODO Allow user to define their own ReportKind?
pub enum ReportKind {
    Error,
    Warning,
    Help,
    Note,
}

// TODO Setting the styling should not be hard coded (and happen later on)
impl ElementLayout for &ReportKind {
    fn element_layout(self) -> Element {
        use crate::tree::{Style, Styled};
        let base_style = Style::new().with_bold();

        match self {
            ReportKind::Error => "error".with_style(base_style.with_fg(Color::Red)),
            ReportKind::Warning => "warning".with_style(base_style.with_fg(Color::Yellow)),
            ReportKind::Help => "help".with_style(base_style.with_fg(Color::Blue)),
            ReportKind::Note => "note".with_style(base_style.with_fg(Color::White)),
        }
    }
}

#[must_use]
pub struct Report<SourceId> {
    kind: ReportKind,
    code: Option<String>,
    message: Vec<tree::Element>,
    /// Annotated section of source code
    view: Option<SourceView<SourceId>>,
    /// Help or note messages
    comments: Vec<(ReportKind, String)>,
}

impl<SourceId> Report<SourceId> {
    // TODO Comments

    pub fn new(kind: ReportKind) -> Self {
        Self {
            kind,
            code: None,
            message: vec![],
            view: None,
            comments: vec![],
        }
    }

    pub fn with_view(mut self, view: SourceView<SourceId>) -> Self {
        self.view = Some(view);
        self
    }

    pub fn set_view(&mut self, view: SourceView<SourceId>) {
        self.view = Some(view);
    }

    pub fn with_message(mut self, message: impl ElementLayout) -> Self {
        self.message.push(message.element_layout());
        self
    }

    pub fn set_message(&mut self, message: impl ElementLayout) {
        self.message.push(message.element_layout());
    }

    pub fn with_code(mut self, code: impl fmt::Display) -> Self {
        self.code = Some(code.to_string());
        self
    }

    pub fn set_code(&mut self, code: impl Into<String>) {
        self.code = Some(code.into());
    }

    pub fn write<B: Backend>(
        &self,
        backend: &mut B,
        cache: &mut impl Cache<SourceId>,
    ) -> Result<(), B::Error> {
        backend.write(self, cache)
    }
}

#[derive(Debug)]
/// Annotated section of source code
pub struct SourceView<Id> {
    source_id: Id,
    location: usize,
    labels: Vec<Label>,
}

impl<Id> SourceView<Id> {
    pub fn new(source_id: Id, location: usize) -> Self {
        Self {
            source_id,
            location,
            labels: vec![],
        }
    }

    pub fn with_label(mut self, label: Label) -> Self {
        self.labels.push(label);
        self
    }

    pub fn with_labels(mut self, labels: impl IntoIterator<Item = Label>) -> Self {
        self.labels.extend(labels);
        self
    }

    pub fn add_label(&mut self, label: Label) {
        self.labels.push(label);
    }

    pub fn add_labels(&mut self, labels: impl IntoIterator<Item = Label>) {
        self.labels.extend(labels);
    }
}

#[derive(Debug)]
pub struct Label {
    span: Span,
    message: Option<Element>,
}

impl Label {
    pub fn new(span: Span) -> Self {
        Self {
            span,
            message: None,
        }
    }

    pub fn with_message(mut self, message: impl ElementLayout) -> Self {
        self.message = Some(message.element_layout());
        self
    }

    pub fn set_message(&mut self, message: impl ElementLayout) {
        self.message = Some(message.element_layout());
    }
}

pub trait Cache<Id: ?Sized> {
    type Error: fmt::Debug;
    // where clause is currently required: https://github.com/rust-lang/rust/issues/87479
    type DisplayedId<'a>: fmt::Display + 'a
    where
        Id: 'a;

    fn fetch(&mut self, id: &Id) -> Result<&str, Self::Error>;

    /// Display the given Id. as a single inline value.
    fn display_id<'a>(&self, id: &'a Id) -> Option<Self::DisplayedId<'a>>;
}

impl<'a> Cache<&'a str> for &[(&'a str, &str)] {
    type Error = ();
    type DisplayedId<'b> = &'b str where &'a str: 'b;

    fn fetch(&mut self, id: &&str) -> Result<&str, Self::Error> {
        if let Some(source) = self
            .iter()
            .find_map(|(key, source)| (key == id).then_some(source))
        {
            Ok(source)
        } else {
            Err(())
        }
    }

    fn display_id(&self, id: &&'a str) -> Option<Self::DisplayedId<'a>> {
        Some(*id)
    }
}

impl Cache<()> for () {
    type Error = ();
    type DisplayedId<'a> = &'a str;

    fn fetch(&mut self, _: &()) -> Result<&str, Self::Error> {
        Err(())
    }

    fn display_id(&self, _: &()) -> Option<Self::DisplayedId<'static>> {
        None
    }
}

#[derive(Debug, Default)]
pub struct FileCache {
    files: HashMap<PathBuf, String>,
}

impl Cache<PathBuf> for FileCache {
    type Error = io::Error;
    type DisplayedId<'a> = std::path::Display<'a>;

    fn fetch(&mut self, id: &PathBuf) -> Result<&str, Self::Error> {
        Ok(self
            .files
            .entry(id.clone())
            .or_insert(fs::read_to_string(id)?))
    }

    fn display_id<'a>(&self, id: &'a PathBuf) -> Option<std::path::Display<'a>> {
        Some(id.display())
    }
}

pub trait Backend {
    type Error;

    fn write<SourceId>(
        &mut self,
        report: &Report<SourceId>,
        cache: &mut impl Cache<SourceId>,
    ) -> Result<(), Self::Error>;
}
