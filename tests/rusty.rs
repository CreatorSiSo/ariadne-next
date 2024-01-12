use ariadne_next::{Ansi, Label, PlainText, Report, ReportKind, SourceView};

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

fn cache() -> Vec<(&'static str, &'static str)> {
    vec![("test.rs", include_str!("./test.rs.txt"))]
}

fn reports() -> [Report<'static, &'static str>; 3] {
    // TODO Add separate Kind/Level for labels?
    // Kind::Add might control the characters "+++", color, ...

    [
        Report::new(ReportKind::Error)
            .with_code("E0412")
            .with_message("cannot find type `Lab` in this scope")
            .with_view(
                SourceView::new("test.rs", 218)
                    .with_label(Label::new(218..221).with_message("not found in this scope")),
            ),
        Report::new(ReportKind::Help)
            .with_message("you might be missing a type parameter")
            .with_view(SourceView::new("test.rs", 218).with_label(Label::new(218..221))),
        Report::new(ReportKind::Error)
            .with_code("E0425")
            .with_message("cannot find value `labels` in this scope")
            .with_view(SourceView::new("test.rs", 1518).with_labels([
                Label::new(1386..1411).with_message("a field by that name exists in `Self`"),
                Label::new(1518..1524),
            ])),
    ]
}

fn render_plainext() -> String {
    let mut backend = PlainText(Vec::new());
    for report in reports() {
        report.write(&mut backend, &mut cache()).unwrap();
    }
    String::from_utf8(backend.0).unwrap()
}

fn render_ansi() -> String {
    let mut backend = Ansi(Vec::new());
    for report in reports() {
        report.write(&mut backend, &mut cache()).unwrap();
    }
    String::from_utf8(backend.0).unwrap()
}

#[test]
fn ansi() {
    let result = render_ansi();
    println!("{result}");
    insta::assert_snapshot!(result);
}

#[test]
fn plaintext() {
    let result = render_plainext();
    println!("{result}");
    insta::assert_snapshot!(result);
}

#[test]
fn stripped_ansi() {
    assert_eq!(
        render_plainext(),
        strip_ansi_escapes::strip_str(render_ansi())
    )
}
