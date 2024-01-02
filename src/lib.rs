use core::fmt;
use std::fmt::Debug;

mod plaintext;
pub use plaintext::PlainText;

pub mod render;
use render::{Element, Inline, IntoElement, TextStyle};

pub type Span = std::ops::Range<usize>;

#[must_use]
pub struct Report<ReportKind: fmt::Debug, LabelKind: fmt::Debug> {
    level: ReportKind,
    code: Option<String>,
    message: Vec<Element>,
    /// Annotated section of source code
    view: Option<SourceView<LabelKind>>,
    /// Help or note messages
    comments: Vec<(ReportKind, String)>,
}

impl<ReportKind: IntoElement + fmt::Debug, LabelKind: fmt::Debug> Report<ReportKind, LabelKind> {
    pub fn new(kind: ReportKind) -> Self {
        Self {
            level: kind,
            code: None,
            message: vec![],
            view: None,
            comments: vec![],
        }
    }

    pub fn with_view(mut self, view: SourceView<LabelKind>) -> Self {
        self.view = Some(view);
        self
    }

    pub fn set_view(&mut self, view: SourceView<LabelKind>) {
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

    pub fn finish(mut self) -> Element {
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
            vstack.push(Element::Inline(Inline::new("   ╭─[<unkown>:255:9]")));
            vstack.push(Element::Inline(Inline::new("")));

            // Find smallest span that encloses all label spans
            let (start, end) = view.labels.iter().fold(
                (view.source.len(), 0),
                |(start, end), Label { span, .. }| (start.min(span.start), end.max(span.end)),
            );

            // TODO Could probably be done better :(
            let index_start = view.source[..start]
                .rfind(|c| c == '\n')
                .map(|i| i + 1)
                .unwrap_or(start);
            let offset_end = &view.source[end..].find(|c| c == '\n').unwrap_or(0);
            let index_end = end + offset_end;

            let content = &view.source[index_start..index_end];
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
                    if label.message.is_empty() {
                        Element::Inline(Inline::new(format!("<marker label>")))
                    } else {
                        Element::Inline(Inline::new(label.message))
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
pub struct SourceView<LabelKind: fmt::Debug> {
    // TODO Should we even store source here?
    source: &'static str,
    labels: Vec<Label<LabelKind>>,
}

impl<Level: fmt::Debug> SourceView<Level> {
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

#[derive(Debug)]
pub struct Label<Kind> {
    kind: Kind,
    span: Span,
    message: String,
}

impl<Level> Label<Level> {
    pub fn new(level: Level, span: Span) -> Self {
        Self {
            kind: level,
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

pub trait Backend {
    type Error;

    fn write(&mut self, element: &Element) -> Result<(), Self::Error>;
}
