use std::{collections::HashMap, fmt::Debug, path::PathBuf};
use std::{fmt, fs, io};

pub mod tree;
use tree::{Element, IntoElement};

mod backends;
pub use backends::{Ansi, PlainText};

pub use yansi::Color;
pub type Span = std::ops::Range<usize>;

// TODO Allow user to define their own ReportKind?
pub enum ReportKind {
    Error,
    Warning,
    Help,
    Note,
}

// TODO Setting the styling should not be hard coded (and happen later on)
impl IntoElement for &ReportKind {
    fn into_element(self) -> Element {
        use crate::tree::*;

        let base_style = TextStyle::new().with_bold();

        Element::Inline(match self {
            ReportKind::Error => {
                Inline::new("error").with_style(base_style.with_fg_color(Color::Red))
            }
            ReportKind::Warning => {
                Inline::new("warning").with_style(base_style.with_fg_color(Color::Yellow))
            }
            ReportKind::Help => {
                Inline::new("help").with_style(base_style.with_fg_color(Color::Blue))
            }
            ReportKind::Note => {
                Inline::new("note").with_style(base_style.with_fg_color(Color::White))
            }
        })
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

    pub fn with_message(mut self, message: impl IntoElement) -> Self {
        self.message.push(message.into_element());
        self
    }

    pub fn set_message(&mut self, message: impl IntoElement) {
        self.message.push(message.into_element());
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
    labels: Vec<Label>,
}

impl<Id> SourceView<Id> {
    pub fn new(source_id: Id) -> Self {
        Self {
            source_id,
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

    pub fn with_message(mut self, message: impl IntoElement) -> Self {
        self.message = Some(message.into_element());
        self
    }

    pub fn set_message(&mut self, message: impl IntoElement) {
        self.message = Some(message.into_element());
    }
}

pub trait Cache<Id: ?Sized>
where
    Self::Error: fmt::Debug,
{
    type Error;

    fn fetch(&mut self, id: &Id) -> Result<&str, Self::Error>;
}

impl Cache<&str> for Vec<(&str, &str)> {
    type Error = ();

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
}

impl Cache<()> for () {
    type Error = ();

    fn fetch(&mut self, _: &()) -> Result<&str, Self::Error> {
        Err(())
    }
}

pub struct FileCache {
    files: HashMap<PathBuf, String>,
}

impl FileCache {
    fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }
}

impl Cache<PathBuf> for FileCache {
    type Error = io::Error;

    fn fetch(&mut self, id: &PathBuf) -> Result<&str, Self::Error> {
        Ok(self
            .files
            .entry(id.clone())
            .or_insert(fs::read_to_string(id)?))
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
