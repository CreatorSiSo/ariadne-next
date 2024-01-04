use super::{layout, Render};
use crate::tree::{Inline, TextStyle};
use crate::{Cache, Report};
use std::io;
use yansi::Paint;

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
        Self::render(&mut self.0, element)
    }
}

impl<W: io::Write> Render for Ansi<W> {
    fn render_inline(inline: Inline) -> impl std::fmt::Display {
        Paint::new(inline.text).with_style((&inline.style).into())
    }
}

impl From<&TextStyle> for yansi::Style {
    fn from(value: &TextStyle) -> Self {
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
