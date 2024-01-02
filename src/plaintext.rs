use itertools::Itertools;

use crate::render::Element;
use std::io;

pub struct PlainText<W: io::Write>(pub W);
pub type PlainTextError = io::Error;

impl<W: io::Write> crate::Backend for PlainText<W> {
    type Error = PlainTextError;

    fn write(&mut self, element: &Element) -> Result<(), Self::Error> {
        let (_, height) = compute_size(element);
        let mut lines = Vec::from_iter((0..height).map(|_| String::new()));

        render_element(&mut lines, element);

        for line in lines {
            writeln!(self.0, "{line}")?;
        }
        Ok(())
    }
}

fn render_element(lines: &mut [String], element: &Element) {
    match element {
        Element::VStack(stack) => {
            let mut start = 0;
            for element in stack {
                let (_, height) = compute_size(element);
                render_element(&mut lines[start..start + height], element);
                start += height;
            }
        }
        Element::HStack(stack) => {
            for element in stack {
                fill_spaces(lines);
                render_element(lines, element);
            }
        }
        this @ Element::Box { content, .. } => {
            let (width, _) = compute_size(this);
            let string: String = content.iter().map(|inline| inline.text.as_str()).collect();
            assert!(!string.contains('\n'));

            fill_spaces(lines);
            for (index, chunk) in string.chars().chunks(width).into_iter().enumerate() {
                let line = &mut lines[index];
                for char in chunk {
                    line.push(char);
                }
            }
        }
        Element::Inline(inline) => lines[0].push_str(&inline.text),
    }
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

#[test]
fn layout() {
    use crate::{Backend, Inline, TextStyle};

    let mut backend = PlainText(Vec::new());
    let element = Element::VStack(vec![
        Element::Inline(Inline::new("test1")),
        Element::HStack(vec![
            Element::Inline(Inline::new("1")),
            Element::Inline(Inline::new(" ")),
            Element::Inline(Inline::new("2")),
            Element::Box {
                content: vec![Inline::new("#_#_#_#__#_#_#_##_#_#_#__#_#_#_#")],
                width: Some(8),
                style: TextStyle::default(),
            },
        ]),
        Element::Inline(Inline::new("test3")),
    ]);
    assert_eq!(compute_size(&element), (11, 6));

    backend.write(&element).unwrap();
    let output = String::from_utf8(backend.0).unwrap();

    // Expected:
    // test1
    // 1 2#_#_#_#_
    //    _#_#_#_#
    //    #_#_#_#_
    //    _#_#_#_#
    // test3
    insta::assert_snapshot!(output);
}
