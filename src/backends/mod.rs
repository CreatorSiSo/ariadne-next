use crate::tree::{Element, Style, Styled};
use crate::{Cache, Label, Report, ReportKind, SourceView, Span, StyledStr};

mod render;
use render::Render;

mod plaintext;
pub use plaintext::PlainText;

mod ansi;
pub use ansi::Ansi;

fn layout_report<SourceId>(
    report: &Report<SourceId>,
    cache: &mut impl Cache<SourceId>,
) -> Styled<Element> {
    let mut vstack: Vec<Styled<Element>> = vec![];

    vstack.push(layout_message(
        report.kind,
        report.code.as_ref(),
        &report.message,
    ));

    for view in &report.views {
        vstack.push(layout_source(view, cache));
    }

    for (kind, message) in &report.comments {
        vstack.push(layout_message(*kind, None, message));
    }

    Element::vstack(vstack).styled(Style::default())
}

fn layout_message(
    kind: ReportKind,
    code: Option<&String>,
    message: &[StyledStr<'_>],
) -> Styled<Element> {
    let mut hstack: Vec<Styled<Element>> = vec![];

    let kind = kind.styled().map(Element::inline);

    let kind_style = kind.style().clone();
    if let Some(code) = &code {
        // TODO Subobptimal should be combined with kind element
        hstack.push(Element::inline(format!("[{code}] ")).styled(kind_style));
    }
    hstack.push(kind);

    hstack.push(Element::inline(": ").styled(Style::default()));

    hstack.extend(
        message
            .iter()
            .map(|styled_text| styled_text.clone().map(Element::inline)),
    );

    Element::hstack(hstack).styled(Style::default())
}

fn layout_source<SourceId>(
    view: &SourceView<SourceId>,
    cache: &mut impl Cache<SourceId>,
) -> Styled<Element> {
    let mut vstack = vec![];

    let name = cache
        .display_id(&view.source_id)
        .map(|id| id.to_string())
        .unwrap_or("<unkown>".into());

    let source = cache.fetch(&view.source_id).unwrap();

    let (lines, cols) = lines_cols(source, view.location, 4);
    vstack.push(Element::inline(format!("[{name}:{lines}:{cols}]")).styled(Style::default()));

    vstack.push(Element::inline("").styled(Style::default()));

    let block = lines_enclosing_spans(source, view.labels.iter().map(|Label { span, .. }| span));
    vstack.extend(
        block
            .lines()
            .map(|line| Element::inline(line).styled(Style::default())),
    );

    vstack.push(Element::inline("").styled(Style::default()));

    // TODO How to build Elements for labels?
    for label in &view.labels {
        vstack.push(
            Element::hstack([
                Element::inline("=> ").styled(Style::default()),
                label
                    .message
                    .as_ref()
                    .map(|parts| {
                        Element::box_(
                            parts.iter().map(|part| part.clone().map(Element::inline)),
                            None,
                        )
                        .styled(Style::default())
                    })
                    .unwrap_or(Element::inline("<empty label>").styled(Style::default())),
                Element::inline(format!(" {:?}", label.span)).styled(Style::default()),
            ])
            .styled(Style::default()),
        )
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
                .styled(Style::default())
            }),
    );

    Element::hstack([
        border.styled(Style::default()),
        Element::vstack(vstack).styled(Style::default()),
    ])
    .styled(Style::default())
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
