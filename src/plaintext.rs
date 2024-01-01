use std::io;

pub struct PlainText<W: io::Write>(pub W);

impl<W: io::Write> crate::Backend for PlainText<W> {
    type Error = ();

    fn write(&mut self, sequence: &crate::Sequence) -> Result<(), Self::Error> {
        todo!()
    }
}
