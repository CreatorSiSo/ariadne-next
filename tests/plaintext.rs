use ariadne_next::{
    render::{Element, Inline, IntoElement, RgbColor, TextStyle},
    Label, PlainText, Report, SourceView,
};
use std::io::Write;

// Goal:
// error[E0412]: cannot find type `Lab` in this scope
//   --> src/lib.rs:10:29
//    |
// 10 |     view: Option<SourceView<Lab>>,
//    |                             ^^^ not found in this scope
//    |
// help: you might be missing a type parameter
//    |
// 5  | pub struct Report<Level, Lab> {
//    |                        +++++

// error[E0425]: cannot find value `labels` in this scope
//   --> src/lib.rs:65:24
//    |
// 60 |     labels: Vec<Label<Level>>,
//    |     ------------------------- a field by that name exists in `Self`
// ...
// 65 |         Self { source, labels }
//    |                        ^^^^^^

// Some errors have detailed explanations: E0412, E0425.
// For more information about an error, try `rustc --explain E0412`.
// error: could not compile `ariadne-rewrite` (lib) due to 2 previous errors

#[test]
fn main() {
    let source = include_str!("./test.rs.txt");
    let mut backend = PlainText(Vec::new());

    Report::new(Level::Error)
        .with_code("E0412")
        .with_message("cannot find type `Lab` in this scope")
        .with_view(
            SourceView::new(&source).with_label(
                Label::new(Level::Error, 218..221).with_message("not found in this scope"),
            ),
        )
        .finish()
        .write(&mut backend)
        .unwrap();

    Report::new(Level::Help)
        .with_message("you might be missing a type parameter")
        .with_view(SourceView::new(&source).with_label(Label::new(
            // TODO Add separate Kind/Level for labels?
            // This could be Kind::Add and control the characters "+++", color, ...
            Level::Help,
            218..221,
        )))
        .finish()
        .write(&mut backend)
        .unwrap();

    writeln!(backend.0).unwrap();

    Report::new(Level::Error)
        .with_code("E0425")
        .with_message("cannot find value `labels` in this scope")
        .with_view(
            SourceView::new(&source).with_labels([
                Label::new(Level::Help, 1386..1411)
                    .with_message("a field by that name exists in `Self`"),
                Label::new(Level::Error, 1518..1524),
            ]),
        )
        .finish()
        .write(&mut backend)
        .unwrap();

    insta::assert_snapshot!(String::from_utf8(backend.0).unwrap());
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
            Level::Error => Inline::new("error")
                .with_style(base_style.with_fg_color(RgbColor::new(237, 61, 61))),
            Level::Warning => Inline::new("warning")
                .with_style(base_style.with_fg_color(RgbColor::new(237, 234, 61))),
            Level::Help => Inline::new("help")
                .with_style(base_style.with_fg_color(RgbColor::new(61, 161, 237))),
        })
    }
}
