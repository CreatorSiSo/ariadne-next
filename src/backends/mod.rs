use std::fmt::Write;

use crate::tree::shortcuts::inline;
use crate::tree::{Element, InlineLayout};
use crate::{Cache, Label, Report, SourceView, Span};

mod plaintext;
pub use plaintext::PlainText;

mod ansi;
pub use ansi::Ansi;
use unicode_width::UnicodeWidthStr;

fn layout<SourceId>(report: &Report<SourceId>, cache: &mut impl Cache<SourceId>) -> Element {
    let mut vstack: Vec<Element> = vec![];

    {
        let mut hstack = vec![];

        let mut kind = report.kind.inline_layout();
        if let Some(code) = &report.code {
            kind.text.write_fmt(format_args!("[{code}]")).unwrap();
        }
        hstack.push(Element::Inline(kind));

        hstack.push(inline(": "));

        hstack.extend(report.message.iter().cloned());
        vstack.push(Element::HStack(hstack));
    }

    if let Some(view) = &report.view {
        vstack.push(layout_view(view, cache));
    }

    Element::VStack(vstack)
}

fn layout_view<SourceId>(view: &SourceView<SourceId>, cache: &mut impl Cache<SourceId>) -> Element {
    let mut vstack = vec![];

    let name = cache
        .display_id(&view.source_id)
        .map(|id| id.to_string())
        .unwrap_or("<unkown>".into());

    let source = cache.fetch(&view.source_id).unwrap();

    let (lines, cols) = lines_cols(source, view.location, 4);
    vstack.push(inline(format!("[{name}:{lines}:{cols}]")));

    vstack.push(inline(""));

    let block = lines_enclosing_spans(source, view.labels.iter().map(|Label { span, .. }| span));
    vstack.extend(block.lines().map(inline));

    vstack.push(inline(""));

    // TODO How to build Elements for labels?
    for label in &view.labels {
        vstack.push(Element::HStack(vec![
            inline("=> "),
            label.message.clone().unwrap_or(inline("<empty label>")),
            inline(format!(" {:?}", label.span)),
        ]))
    }

    let border = Element::VStack(
        vstack
            .iter()
            .enumerate()
            .map(|(i, _)| i)
            .chain(Some(vstack.len()))
            .map(|i| {
                dbg!(i);
                inline(match i {
                    0 => "   ╭─",
                    _ if i == vstack.len() => "───╯ ",
                    _ => "   │ ",
                })
            })
            .collect(),
    );

    Element::HStack(vec![border, Element::VStack(vstack)])
}

fn compute_size(element: &Element) -> (usize, usize) {
    match element {
        Element::VStack(stack) => stack
            .iter()
            .map(compute_size)
            .fold((0, 0), |(width, height), (w, h)| (width.max(w), height + h)),
        Element::HStack(stack) => stack
            .iter()
            .map(compute_size)
            .fold((0, 0), |(width, height), (w, h)| (width + w, height.max(h))),
        Element::Box { content, width, .. } => {
            let len: usize = content.iter().map(|inline| inline.text.len()).sum();
            if let Some(width) = width {
                (*width, len.div_ceil(*width))
            } else {
                (len, 1)
            }
        }
        Element::Inline(inline) => (
            inline.text.len(),
            1, /* TODO Set this to 0 when text is empty? */
        ),
    }
}

fn fill_spaces(lines: &mut [String]) {
    let max_width = lines.iter().map(|line| line.width()).max().unwrap_or(0);
    for line in lines {
        line.push_str(&" ".repeat(max_width - line.width()));
    }
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
