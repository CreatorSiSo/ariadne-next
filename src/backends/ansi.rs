use super::{layout_report, Render};
use crate::{Cache, Report};
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
        let element = layout_report(report, cache);
        Self::render(&mut self.0, &element)
    }
}

impl<W: io::Write> Render for Ansi<W> {
    fn write_style_prefix(string: &mut String, style: &crate::Style) {
        yansi::Style::from(style).fmt_prefix(string).unwrap();
    }

    fn write_style_suffix(string: &mut String, style: &crate::Style) {
        yansi::Style::from(style).fmt_suffix(string).unwrap();
    }
}

impl From<&crate::Style> for yansi::Style {
    fn from(value: &crate::Style) -> Self {
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
