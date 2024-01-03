use crate::{
    tree::{Element, TextStyle},
    Cache, Report,
};
use itertools::Itertools;
use std::io;
use yansi::{Paint, Style};

pub struct Ansi<W: io::Write>(pub W);
pub type AnsiError = io::Error;

impl<W: io::Write> crate::Backend for Ansi<W> {
    type Error = AnsiError;

    fn write<SourceId>(
        &mut self,
        report: &Report<SourceId>,
        cache: &mut impl Cache<SourceId>,
    ) -> Result<(), Self::Error> {
        let element = super::layout(report, cache);
        self.render(&element)
    }
}

impl<W: io::Write> Ansi<W> {
    fn render(&mut self, element: &Element) -> Result<(), AnsiError> {
        let (_, height) = compute_size(element);
        let mut lines = Vec::from_iter((0..height).map(|_| String::new()));

        render_element(&mut lines, &element);

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
        Element::Inline(inline) => {
            let styled = Paint::new(&inline.text).with_style((&inline.style).into());
            lines[0].push_str(&styled.to_string())
        }
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

impl From<&TextStyle> for Style {
    fn from(value: &TextStyle) -> Self {
        let mut style = Style::default();
        if let Some(fg_color) = &value.fg_color {
            style = style.fg(*fg_color);
        }
        style
    }
}
