use crate::render::{Element, VStack};
use itertools::Itertools;
use std::io;

pub struct PlainText<W: io::Write>(pub W);

impl<W: io::Write> crate::Backend for PlainText<W> {
    type Error = io::Error;

    fn write(&mut self, stack: &VStack) -> Result<(), Self::Error> {
        let mut lines = Lines::new();

        for element in &stack.elements {
            render_element(&mut lines, element);

            for line in lines.iter() {
                writeln!(self.0, "{line}").unwrap();
            }
            lines.clear();
        }

        Ok(())
    }
}

fn render_element(lines: &mut Lines, element: &Element) {
    match element {
        Element::HStack(elements) => {
            for element in elements {
                render_element(lines, element);
            }
        }
        Element::Container {
            content,
            width,
            height,
            style,
        } => {
            let longest_len = lines.iter().map(|line| line.len()).max().unwrap_or(0);
            let flat_string: String = content.iter().map(|inline| inline.text.as_str()).collect();
            let width = width.unwrap_or(usize::MAX);

            let mut w = 1;
            let iter = flat_string
                .split_inclusive(|c| {
                    let split = c == '\n' || w == width;
                    if split {
                        w = 1;
                    } else {
                        w += 1;
                    }
                    split
                })
                .enumerate();

            for (index, inner_line) in iter {
                let line = lines.get_mut(index);
                // Append spaces to lines that are shorter than the longest line
                line.push_str(&" ".repeat(longest_len - line.len()));

                // TODO Remove this when SourceView just generates a VStack
                line.push_str(inner_line.trim_end_matches(|c| c == '\n'));
            }
        }
        Element::Inline(inline) => lines.get_mut(0).push_str(&inline.text),
    }
}

struct Lines {
    lines: Vec<String>,
    len: usize,
}

impl Lines {
    fn new() -> Self {
        Self {
            lines: vec![],
            len: 0,
        }
    }

    fn get_mut(&mut self, index: usize) -> &mut String {
        while index >= self.lines.len() {
            self.lines.push(String::new());
        }
        return self.lines.get_mut(index).unwrap();
    }

    fn clear(&mut self) {
        self.lines.clear()
    }

    fn iter(&self) -> std::slice::Iter<String> {
        self.lines.iter()
    }
}

#[test]
fn layout() {
    use crate::{Backend, Inline, TextStyle};

    let mut backend = PlainText(Vec::new());

    backend
        .write(&VStack {
            elements: vec![
                Element::Inline(Inline::new("test1")),
                Element::HStack(vec![
                    Element::Inline(Inline::new("1")),
                    Element::Inline(Inline::new(" ")),
                    Element::Inline(Inline::new("2")),
                    Element::Container {
                        content: vec![Inline::new("#_#_#_#__#_#_#_##_#_#_#__#_#_#_#")],
                        width: Some(8),
                        height: Some(8),
                        style: TextStyle::default(),
                    },
                ]),
                Element::Inline(Inline::new("test3")),
            ],
        })
        .unwrap();

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
