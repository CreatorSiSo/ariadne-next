use super::{layout, Render};
use crate::{tree::Style, Cache, Report};
use std::io;

pub struct Ansi<W: io::Write>(pub W);
pub type AnsiError = io::Error;

impl<W: io::Write> crate::Backend for Ansi<W> {
    type Error = AnsiError;

    fn write<SourceId>(
        &mut self,
        report: &Report<SourceId>,
        cache: &mut impl Cache<SourceId>,
    ) -> Result<(), Self::Error> {
        let element = layout(report, cache);
        Self::render(&mut self.0, &element)
    }
}

impl<W: io::Write> Render for Ansi<W> {
    fn write_style_prefix(string: &mut String, style: &Style) {
        yansi::Style::from(style).fmt_prefix(string).unwrap();
    }

    fn write_style_suffix(string: &mut String, style: &Style) {
        yansi::Style::from(style).fmt_suffix(string).unwrap();
    }
}

impl From<&crate::tree::Style> for yansi::Style {
    fn from(value: &crate::tree::Style) -> Self {
        let mut style = yansi::Style::default();
        if let Some(fg_color) = &value.fg_color {
            style = style.fg(*fg_color);
        }
        if let Some(bg_color) = &value.bg_color {
            style = style.bg(*bg_color);
        }
        if value.is_bold() {
            style = style.bold();
        }
        if value.is_italic() {
            style = style.italic();
        }
        style
    }
}
