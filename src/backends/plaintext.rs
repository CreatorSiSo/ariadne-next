use super::{layout, Render};
use crate::{Cache, Report};
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
        Self::render(&mut self.0, &element)
    }
}

impl<W: io::Write> Render for PlainText<W> {}
