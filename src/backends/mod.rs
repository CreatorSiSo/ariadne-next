use crate::tree::{Element, Inline, IntoElement};
use crate::{Cache, Label, Report};

mod plaintext;
pub use plaintext::PlainText;

mod ansi;
pub use ansi::Ansi;

fn layout<SourceId>(report: &Report<SourceId>, cache: &mut impl Cache<SourceId>) -> Element {
    let mut vstack: Vec<Element> = vec![];

    {
        let mut first_line = vec![report.kind.into_element()];
        if let Some(code) = &report.code {
            // TODO Should the code use the style of self.level?
            first_line.push(format!("[{code}]").into_element());
        }
        first_line.push(": ".into_element());
        first_line.extend(report.message.iter().cloned());

        vstack.push(Element::HStack(first_line));
    }

    if let Some(view) = &report.view {
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
        for label in &view.labels {
            vstack.push(Element::HStack(vec![
                Element::Inline(Inline::new("=> ")),
                if let Some(message) = &label.message {
                    message.clone()
                } else {
                    Element::Inline(Inline::new("<empty label>"))
                },
                Element::Inline(Inline::new(format!(" {:?}", label.span))),
            ]))
        }
    }

    Element::VStack(vstack)
}

fn compute_size(element: &Element) -> (usize, usize) {
    match element {
        Element::VStack(stack) => stack
            .iter()
            .map(|element| compute_size(element))
            .fold((0, 0), |(width, height), (w, h)| (width.max(w), height + h)),
        Element::HStack(stack) => stack
            .iter()
            .map(|element| compute_size(element))
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
