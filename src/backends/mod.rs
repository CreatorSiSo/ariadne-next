use crate::tree::{Element, Layout, Styled};
use crate::{Cache, Label, Report, SourceView, Span};

mod render;
pub(self) use render::Render;

mod plaintext;
pub use plaintext::PlainText;

mod ansi;
pub use ansi::Ansi;

fn layout<SourceId>(report: &Report<SourceId>, cache: &mut impl Cache<SourceId>) -> Element {
    let mut vstack: Vec<Element> = vec![];

    {
        let mut hstack = vec![];

        let kind = report.kind.layout();
        let kind_style = kind.style().clone();
        hstack.push(kind);
        if let Some(code) = &report.code {
            // TODO Subobptimal should be combined with kind element
            hstack.push(format!("[{code}]").with_style(kind_style));
        }

        hstack.push(Element::inline(": "));

        hstack.extend(report.message.iter().cloned());
        vstack.push(Element::hstack(hstack));
    }

    if let Some(view) = &report.view {
        vstack.push(layout_source(view, cache));
    }

    Element::vstack(vstack)
}

fn layout_source<SourceId>(
    view: &SourceView<SourceId>,
    cache: &mut impl Cache<SourceId>,
) -> Element {
    let mut vstack = vec![];

    let name = cache
        .display_id(&view.source_id)
        .map(|id| id.to_string())
        .unwrap_or("<unkown>".into());

    let source = cache.fetch(&view.source_id).unwrap();

    let (lines, cols) = lines_cols(source, view.location, 4);
    vstack.push(Element::inline(format!("[{name}:{lines}:{cols}]")));

    vstack.push(Element::inline(""));

    let block = lines_enclosing_spans(source, view.labels.iter().map(|Label { span, .. }| span));
    vstack.extend(block.lines().map(Element::inline));

    vstack.push(Element::inline(""));

    // TODO How to build Elements for labels?
    for label in &view.labels {
        vstack.push(Element::hstack([
            Element::inline("=> "),
            label
                .message
                .clone()
                .unwrap_or(Element::inline("<empty label>")),
            Element::inline(format!(" {:?}", label.span)),
        ]))
    }

    // TODO Do not use hardcoded characters
    let border = Element::vstack(
        vstack
            .iter()
            .enumerate()
            .map(|(i, _)| i)
            .chain(Some(vstack.len()))
            .map(|i| {
                Element::inline(match i {
                    0 => "   ╭─",
                    _ if i == vstack.len() => "───╯ ",
                    _ => "   │ ",
                })
            }),
    );

    Element::hstack([border, Element::vstack(vstack)])
}

fn lines_cols(source: &str, location: usize, tab_width: u32) -> (usize, u32) {
    let source_before = &source[..location];
    let lines = source_before.lines().count();
    let cols = source_before
        .lines()
        .last()
        .map(|line| {
            1 + line
                .chars()
                .map(|c| match c {
                    '\t' => tab_width,
                    _ => 1,
                })
                .sum::<u32>()
        })
        .unwrap_or(1);

    (lines, cols)
}

fn lines_enclosing_spans<'a>(source: &str, spans: impl Iterator<Item = &'a Span>) -> &str {
    // Find smallest span that encloses all spans
    let (start, end) = spans.fold((source.len(), 0), |(start, end), span| {
        (start.min(span.start), end.max(span.end))
    });

    let first_line_start = source[..start]
        .rfind(|c| c == '\n')
        .map(|i| i + 1)
        .unwrap_or(start);
    let offset_end = &source[end..].find(|c| c == '\n').unwrap_or(0);
    let last_line_end = end + offset_end;

    &source[first_line_start..last_line_end]
}
