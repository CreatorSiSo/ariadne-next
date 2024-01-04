use super::{compute_size, fill_spaces, layout};
use crate::{tree::Element, Cache, Report};
use itertools::Itertools;
use std::io;

pub struct PlainText<W: io::Write>(pub W);
pub type PlainTextError = io::Error;

impl<W: io::Write> crate::Backend for PlainText<W> {
    type Error = PlainTextError;

    fn write<SourceId>(
        &mut self,
        report: &Report<SourceId>,
        cache: &mut impl Cache<SourceId>,
    ) -> Result<(), Self::Error> {
        let element = layout(report, cache);
        self.render(&element)
    }
}

impl<W: io::Write> PlainText<W> {
    fn render(&mut self, element: &Element) -> Result<(), PlainTextError> {
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

#[test]
fn test_layout() {
    use crate::tree::{shortcuts::inline, Inline, TextStyle};

    let mut backend = PlainText(Vec::new());
    let element = Element::VStack(vec![
        inline("test1"),
        Element::HStack(vec![
            inline("1"),
            inline(" "),
            inline("2"),
            Element::Box {
                content: vec![Inline::new("#_#_#_#__#_#_#_##_#_#_#__#_#_#_#")],
                width: Some(8),
                style: TextStyle::default(),
            },
        ]),
        Element::Inline(Inline::new("test3")),
    ]);
    assert_eq!(compute_size(&element), (11, 6));

    backend.render(&element).unwrap();
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
