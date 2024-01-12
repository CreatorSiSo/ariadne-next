use ariadne_next::{Ansi, Color, Label, Report, ReportKind, SourceView, StyleExt};

fn main() {
    let mut backend = Ansi(std::io::stdout().lock());

    let a = Color::Cyan;
    let b = Color::Green;
    let c = Color::Yellow;

    Report::new(ReportKind::Error)
        .with_code(3)
        .with_message("Cannot add types Nat and Str")
        .with_view(
            SourceView::new("b.tao", 10)
                .with_label(
                    Label::new(10..14)
                        .with_message(["This is of type ".into(), "Nat".fg(a)])
                        .with_color(a),
                )
                .with_label(
                    Label::new(17..20)
                        .with_message(["This is of type ".into(), "Str".fg(b)])
                        .with_color(b),
                )
                .with_label(
                    Label::new(15..16)
                        .with_message([
                            "Nat".fg(a),
                            " and ".into(),
                            "Str".fg(b),
                            " undergo addition here".into(),
                        ])
                        .with_color(c),
                    // .with_order(10),
                ),
        )
        .with_view(
            SourceView::new("a.tao", 4).with_label(
                Label::new(4..8)
                    .with_message([
                        "Original definition of ".into(),
                        "five".fg(a),
                        " is here".into(),
                    ])
                    .with_color(a),
            ),
        )
        .with_comment(
            ReportKind::Note,
            [
                "Nat".fg(a),
                " is a number and can only be added to other numbers".into(),
            ],
        )
        .write(
            &mut backend,
            &mut vec![
                ("a.tao", include_str!("a.tao")),
                ("b.tao", include_str!("b.tao")),
            ],
        )
        .unwrap();
}
