use ariadne_next::tree::{Element, Inline, IntoElement, TextStyle};
use ariadne_next::{Ansi, Color, Label, Report, SourceView};
use std::io::{stdout, Write};

fn main() {
    let mut cache = vec![("test.rs", include_str!("./test.rs.txt"))];
    let mut backend = Ansi(stdout().lock());

    Report::new(Level::Error)
        .with_code("E0412")
        .with_message("cannot find type `Lab` in this scope")
        .with_view(
            SourceView::new("test.rs")
                .with_label(Label::new(218..221).with_message("not found in this scope")),
        )
        .finish(&mut cache)
        .write(&mut backend)
        .unwrap();

    Report::new(Level::Help)
        .with_message("you might be missing a type parameter")
        .with_view(SourceView::new("test.rs").with_label(Label::new(218..221)))
        .finish(&mut cache)
        .write(&mut backend)
        .unwrap();

    writeln!(backend.0).unwrap();

    Report::new(Level::Error)
        .with_code("E0425")
        .with_message("cannot find value `labels` in this scope")
        .with_view(SourceView::new("test.rs").with_labels([
            Label::new(1386..1411).with_message("a field by that name exists in `Self`"),
            Label::new(1518..1524),
        ]))
        .finish(&mut cache)
        .write(&mut backend)
        .unwrap();
}

#[derive(Debug)]
enum Level {
    Error,
    Warning,
    Help,
}

impl IntoElement for Level {
    fn into_element(self) -> Element {
        let base_style = TextStyle::new().with_bold();

        Element::Inline(match self {
            Level::Error => {
                Inline::new("error").with_style(base_style.with_fg_color(Color::RGB(237, 61, 61)))
            }
            Level::Warning => Inline::new("warning")
                .with_style(base_style.with_fg_color(Color::RGB(237, 234, 61))),
            Level::Help => {
                Inline::new("help").with_style(base_style.with_fg_color(Color::RGB(61, 161, 237)))
            }
        })
    }
}
