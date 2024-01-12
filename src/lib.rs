use std::borrow::Cow;
use std::{collections::HashMap, fmt::Debug, path::PathBuf};
use std::{fmt, fs, io};

pub mod tree;
use tree::{Style, Styled};

mod backends;
pub use backends::{Ansi, PlainText};

pub type Color = yansi::Color;
pub type Span = std::ops::Range<usize>;

#[derive(Debug, Clone, Copy)]
// TODO Allow user to define their own ReportKind?
pub enum ReportKind {
    Error,
    Warning,
    Help,
    Note,
}

// TODO Setting the styling should not be hard coded (and maybe happen later on)
impl ReportKind {
    fn styled(&self) -> Styled<Cow<'static, str>> {
        let base_style = Style::new().bold();

        match self {
            ReportKind::Error => Styled::new("error".into(), base_style.fg(Color::Red)),
            ReportKind::Warning => Styled::new("warning".into(), base_style.fg(Color::Yellow)),
            ReportKind::Help => Styled::new("help".into(), base_style.fg(Color::Blue)),
            ReportKind::Note => Styled::new("note".into(), base_style.fg(Color::White)),
        }
    }
}

#[must_use]
pub struct Report<'a, SourceId> {
    kind: ReportKind,
    code: Option<String>,
    message: Vec<StyledStr<'a>>,
    /// Annotated section of source code
    views: Vec<SourceView<'a, SourceId>>,
    /// Help or note messages
    comments: Vec<(ReportKind, Vec<StyledStr<'a>>)>,
}

impl<'a, SourceId> Report<'a, SourceId> {
    // TODO Comments

    pub fn new(kind: ReportKind) -> Self {
        Self {
            kind,
            code: None,
            message: vec![],
            views: vec![],
            comments: vec![],
        }
    }

    pub fn with_message(mut self, message: impl StyledText<'a>) -> Self {
        self.message = message.parts_vec();
        self
    }

    pub fn set_message(&mut self, message: impl StyledText<'a>) {
        self.message = message.parts_vec();
    }

    pub fn with_code(mut self, code: impl fmt::Display) -> Self {
        self.code = Some(code.to_string());
        self
    }

    pub fn set_code(&mut self, code: impl Into<String>) {
        self.code = Some(code.into());
    }

    pub fn with_view(mut self, view: SourceView<'a, SourceId>) -> Self {
        self.views.push(view);
        self
    }

    pub fn add_view(&mut self, view: SourceView<'a, SourceId>) {
        self.views.push(view);
    }

    pub fn with_comment(mut self, kind: ReportKind, comment: impl StyledText<'a>) -> Self {
        self.comments.push((kind, comment.parts_vec()));
        self
    }

    pub fn set_comment(&mut self, kind: ReportKind, comment: impl StyledText<'a>) {
        self.comments.push((kind, comment.parts_vec()));
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
pub struct SourceView<'a, Id> {
    source_id: Id,
    location: usize,
    labels: Vec<Label<'a>>,
}

impl<'a, Id> SourceView<'a, Id> {
    pub fn new(source_id: Id, location: usize) -> Self {
        Self {
            source_id,
            location,
            labels: vec![],
        }
    }

    pub fn with_label(mut self, label: Label<'a>) -> Self {
        self.labels.push(label);
        self
    }

    pub fn with_labels(mut self, labels: impl IntoIterator<Item = Label<'a>>) -> Self {
        self.labels.extend(labels);
        self
    }

    pub fn add_label(&mut self, label: Label<'a>) {
        self.labels.push(label);
    }

    pub fn add_labels(&mut self, labels: impl IntoIterator<Item = Label<'a>>) {
        self.labels.extend(labels);
    }
}

#[derive(Debug)]
pub struct Label<'a> {
    span: Span,
    message: Option<Vec<Styled<Cow<'a, str>>>>,
    color: Color,
}

impl<'a> Label<'a> {
    pub fn new(span: Span) -> Self {
        Self {
            span,
            message: None,
            color: Color::Unset,
        }
    }

    pub fn with_message(mut self, message: impl StyledText<'a>) -> Self {
        self.message = Some(message.parts_vec());
        self
    }

    pub fn set_message(&mut self, message: impl StyledText<'a>) {
        self.message = Some(message.parts_vec());
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
}

type StyledStr<'a> = Styled<Cow<'a, str>>;

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

impl<Id, C: Cache<Id>> Cache<Id> for &mut C {
    type Error = C::Error;
    type DisplayedId<'a> = C::DisplayedId<'a> where Id: 'a;

    fn fetch(&mut self, id: &Id) -> Result<&str, Self::Error> {
        C::fetch(self, id)
    }

    fn display_id<'b>(&self, id: &'b Id) -> Option<Self::DisplayedId<'b>> {
        C::display_id(self, id)
    }
}

impl<'a> Cache<&'a str> for Vec<(&'a str, &'a str)> {
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
