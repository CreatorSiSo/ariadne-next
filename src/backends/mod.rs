use std::fmt::Write;

use crate::tree::shortcuts::inline;
use crate::tree::{Element, InlineLayout};
use crate::{Cache, Label, Report};

mod plaintext;
pub use plaintext::PlainText;

mod ansi;
pub use ansi::Ansi;

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
        let name = cache
            .display_id(&view.source_id)
            .map(|id| id.to_string())
            .unwrap_or("<unkown>".into());

        let source = cache.fetch(&view.source_id).unwrap();

        let (lines, cols) = lines_cols(source, view.location, 4);
        vstack.push(inline(format!("   ╭─[{name}:{lines}:{cols}]")));
        vstack.push(inline(""));

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
            inline("     "),
            Element::VStack(content.lines().map(inline).collect()),
        ]));

        vstack.push(inline(""));

        // TODO How to build Elements for labels?
        for label in &view.labels {
            vstack.push(Element::HStack(vec![
                inline("=> "),
                if let Some(message) = &label.message {
                    message.clone()
                } else {
                    inline("<empty label>")
                },
                inline(format!(" {:?}", label.span)),
            ]))
        }
    }

    Element::VStack(vstack)
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
    let width = lines.iter().map(|line| line.len()).max().unwrap_or(0);
    for line in lines {
        line.push_str(&" ".repeat(width - line.len()));
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
