use crate::tree::{Element, Inline};
use itertools::Itertools;
use std::io;
use unicode_width::UnicodeWidthStr;

pub(super) trait Render {
    fn render(writer: &mut impl io::Write, element: Element) -> Result<(), io::Error> {
        let (_, height) = compute_size(&element);
        let mut lines = Vec::from_iter((0..height).map(|_| String::new()));

        Self::render_element(&mut lines, element);

        for line in lines {
            writeln!(writer, "{line}")?;
        }
        Ok(())
    }

    fn render_element(lines: &mut [String], element: Element) {
        match element {
            Element::VStack(elements) => Self::render_vstack(lines, elements),
            Element::HStack(elements) => Self::render_hstack(lines, elements),
            ref this @ Element::Box { ref content, .. } => Self::render_box(lines, this, content),
            Element::Inline(inline) => lines[0].push_str(&Self::render_inline(inline).to_string()),
        }
    }

    fn render_vstack(lines: &mut [String], elements: Vec<Element>) {
        let mut start = 0;
        for element in elements {
            let (_, height) = compute_size(&element);
            Self::render_element(&mut lines[start..start + height], element);
            start += height;
        }
    }

    fn render_hstack(lines: &mut [String], elements: Vec<Element>) {
        for element in elements {
            fill_spaces(lines);
            Self::render_element(lines, element);
        }
    }

    fn render_box(lines: &mut [String], element: &Element, content: &[Inline]) {
        let string: String = content.iter().map(|inline| inline.text.as_str()).collect();
        debug_assert!(!string.contains('\n'));

        fill_spaces(lines);
        let (width, _) = compute_size(element);
        for (index, chunk) in string.chars().chunks(width).into_iter().enumerate() {
            let line = &mut lines[index];
            for char in chunk {
                line.push(char);
            }
        }
    }

    fn render_inline(inline: Inline) -> impl std::fmt::Display;
}

fn fill_spaces(lines: &mut [String]) {
    let max_width = lines.iter().map(|line| line.width()).max().unwrap_or(0);
    for line in lines {
        line.push_str(&" ".repeat(max_width - line.width()));
    }
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

#[test]
fn test_rendering() {
    use crate::tree::{shortcuts::inline, Element, Inline, TextStyle};

    struct TestBackend;
    impl Render for TestBackend {
        fn render_inline(inline: Inline) -> impl std::fmt::Display {
            inline.text
        }
    }

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

    let mut writer = Vec::new();
    TestBackend::render(&mut writer, element).unwrap();
    let output = String::from_utf8(writer).unwrap();

    // Expected:
    // test1
    // 1 2#_#_#_#_
    //    _#_#_#_#
    //    #_#_#_#_
    //    _#_#_#_#
    // test3
    insta::assert_snapshot!(output);
}
