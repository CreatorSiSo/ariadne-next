use std::{collections::HashMap, fmt::Debug, path::PathBuf};
use std::{fmt, fs, io};

mod plaintext;
pub use plaintext::PlainText;

mod ansi;
pub use ansi::Ansi;

pub mod tree;
use tree::{Element, IntoElement};

pub use yansi::Color;
pub type Span = std::ops::Range<usize>;

#[must_use]
pub struct Report<ReportKind, SourceId> {
    level: ReportKind,
    code: Option<String>,
    message: Vec<tree::Element>,
    /// Annotated section of source code
    view: Option<SourceView<SourceId>>,
    /// Help or note messages
    comments: Vec<(ReportKind, String)>,
}

impl<ReportKind: IntoElement, SourceId> Report<ReportKind, SourceId> {
    pub fn new(kind: ReportKind) -> Self {
        Self {
            level: kind,
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

    // TODO Comments

    pub fn finish(mut self, cache: &mut impl Cache<SourceId>) -> tree::Element {
        use tree::*;

        let mut vstack: Vec<Element> = vec![];

        {
            let mut first_line = vec![self.level.into_element()];
            if let Some(code) = self.code {
                // TODO Should the code use the style of self.level?
                first_line.push(format!("[{code}]").into_element());
            }
            first_line.push(": ".into_element());
            first_line.append(&mut self.message);

            vstack.push(Element::HStack(first_line));
        }

        if let Some(view) = self.view {
            let source = cache.fetch(&view.source_id).unwrap();

            vstack.push(Element::Inline(Inline::new("   ╭─[<unkown>:255:9]")));
            vstack.push(Element::Inline(Inline::new("")));

            // Find smallest span that encloses all label spans
            let (start, end) = view
                .labels
                .iter()
                .fold((source.len(), 0), |(start, end), Label { span, .. }| {
                    (start.min(span.start), end.max(span.end))
                });

            // TODO Could probably be done better :(
            let index_start = source[..start]
                .rfind(|c| c == '\n')
                .map(|i| i + 1)
                .unwrap_or(start);
            let offset_end = &source[end..].find(|c| c == '\n').unwrap_or(0);
            let index_end = end + offset_end;

            let content = &source[index_start..index_end];
            // let width = content.lines().map(|line| line.len()).max().unwrap();

            vstack.push(Element::HStack(vec![
                Element::Inline(Inline::new("     ")),
                Element::VStack(content.lines().map(|line| line.into_element()).collect()),
            ]));

            vstack.push(Element::Inline(Inline::new("")));

            // TODO How to build Elements for labels?
            for label in view.labels {
                vstack.push(Element::HStack(vec![
                    Element::Inline(Inline::new("=> ")),
                    if let Some(message) = label.message {
                        message
                    } else {
                        Element::Inline(Inline::new("<empty label>"))
                    },
                    Element::Inline(Inline::new(format!(" {:?}", label.span))),
                ]))
            }
        }

        Element::VStack(vstack)
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

    fn write(&mut self, element: &tree::Element) -> Result<(), Self::Error>;
}
