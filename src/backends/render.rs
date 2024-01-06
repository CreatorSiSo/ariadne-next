use crate::tree::{Element, Style};
use std::io;
use unicode_segmentation::{GraphemeIndices, UnicodeSegmentation};
use unicode_width::UnicodeWidthStr;

pub(super) trait Render {
    fn render(writer: &mut impl io::Write, element: &Element) -> Result<(), io::Error> {
        let (_, height) = compute_size(element);
        let mut lines = Vec::from_iter((0..height).map(|_| String::new()));

        Self::render_element(&mut lines, element);

        for line in lines {
            writeln!(writer, "{line}")?;
        }
        Ok(())
    }

    fn render_element(lines: &mut [String], element: &Element) {
        Self::write_style_prefix(lines.first_mut().unwrap(), element.style());
        match element {
            Element::VStack {
                children: elements, ..
            } => Self::render_vstack(lines, elements),
            Element::HStack {
                children: elements, ..
            } => Self::render_hstack(lines, elements),
            Element::Box {
                children: elements, ..
            } => Self::render_box(lines, element, elements),
            Element::Inline { text, .. } => lines[0].push_str(text),
        }
        Self::write_style_suffix(lines.last_mut().unwrap(), element.style());
    }

    fn render_vstack(lines: &mut [String], elements: &[Element]) {
        let mut start = 0;
        for element in elements {
            let (_, height) = compute_size(element);
            Self::render_element(&mut lines[start..start + height], element);
            start += height;
        }
    }

    fn render_hstack(lines: &mut [String], elements: &[Element]) {
        for element in elements {
            fill_spaces(lines);
            Self::render_element(lines, element);
        }
    }

    fn render_box(lines: &mut [String], box_: &Element, elements: &[Element]) {
        fill_spaces(lines);

        let (bow_width, _) = compute_size(box_);
        for element in elements {
            if let Element::Inline { text, style } = element {
                Self::write_style_prefix(lines.first_mut().unwrap(), style);
                for (index, chunk) in WidthChunks::new(text, bow_width).enumerate() {
                    lines[index].push_str(chunk);
                }
                Self::write_style_suffix(lines.last_mut().unwrap(), style);
                continue;
            }
            Self::render_element(lines, element);
        }
    }

    // TODO Not sure whether this will actually hold up for html
    fn write_style_prefix(_string: &mut String, _style: &Style) {}
    fn write_style_suffix(_string: &mut String, _style: &Style) {}
}

fn fill_spaces(lines: &mut [String]) {
    let max_width = lines.iter().map(|line| line.width()).max().unwrap_or(0);
    for line in lines {
        line.push_str(&" ".repeat(max_width - line.width()));
    }
}

fn compute_size(element: &Element) -> (usize, usize) {
    match element {
        Element::VStack {
            children: elements, ..
        } => elements
            .iter()
            .map(compute_size)
            .fold((0, 0), |(width, height), (w, h)| (width.max(w), height + h)),
        Element::HStack {
            children: elements, ..
        } => elements
            .iter()
            .map(compute_size)
            .fold((0, 0), |(width, height), (w, h)| (width + w, height.max(h))),
        Element::Box {
            children: elements,
            width: max_width,
            ..
        } => {
            let Some(box_width) = max_width else {
                // No width is set, so nothing will be wrapped
                return elements
                    .iter()
                    .map(compute_size)
                    .reduce(|(width_sum, height_max), (width, height)| {
                        (width_sum + width, height_max.max(height))
                    })
                    .unwrap_or((0, 1));
            };
            let mut box_height = 1;

            let mut row_width = 0;
            let mut row_height = 1;

            for element in elements {
                let (elem_width, elem_height) = compute_size(element);

                if matches!(element, Element::Inline { .. }) {
                    // Break text up over multiple rows
                    box_height += (row_width + elem_width - box_width) / *box_width;
                    row_width = (row_width + elem_width) % *box_width;
                    continue;
                }

                if row_width + elem_width > *box_width {
                    // Render element in next row
                    box_height += row_height;
                    row_width = elem_width;
                    row_height = elem_height;
                } else {
                    // Render element on the same row
                    row_height = row_height.max(elem_height);
                    row_width += elem_width;
                }
            }

            (*box_width, box_height)
        }
        Element::Inline { text, .. } => (text.width(), 1),
    }
}

#[must_use]
/// Splits a &[`str`] into chunks,
/// so that the unicode width is smaller or equal than the provided one
struct WidthChunks<'a> {
    /// How many chunks this iterator will produce.
    len: usize,
    /// The maximum width of one chunk.
    width: usize,
    /// The starting byte index of the next chunk.
    start: usize,
    /// The underlying str.
    slice: &'a str,
    /// Iterator over its graphames,
    /// so we dont have to reconstruct it on every next call.
    graphemes: GraphemeIndices<'a>,
}

impl<'a> WidthChunks<'a> {
    fn new(slice: &'a str, width: usize) -> Self {
        Self {
            len: slice.width().div_ceil(width),
            width,
            start: 0,
            slice,
            graphemes: slice.grapheme_indices(true),
        }
    }
}

impl<'a> Iterator for WidthChunks<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start >= self.slice.len() {
            return None;
        }

        let mut width = 0;
        let mut end = self.start;
        for (index, grapheme) in &mut self.graphemes {
            width += grapheme.width();
            end = index + grapheme.len();
            if width >= self.width {
                break;
            }
        }

        let result = &self.slice[self.start..end];
        self.start = end;
        Some(result)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // TODO Is this actually correct?
        (self.len, Some(self.len))
    }
}

#[test]
fn test_width_chunks() {
    // Fix for type inference
    let vec = |slice| {
        let mut vec = Vec::<&str>::new();
        vec.extend_from_slice(slice);
        vec
    };

    assert_eq!(WidthChunks::new("", 4).collect::<Vec<&str>>(), vec(&[]));

    assert_eq!(WidthChunks::new("test", 4).collect::<Vec<_>>(), ["test"]);

    assert_eq!(
        WidthChunks::new("12345─", 5).collect::<Vec<_>>(),
        ["12345", "─"]
    );

    assert_eq!(
        WidthChunks::new("╯││ ──", 5).collect::<Vec<_>>(),
        ["╯││ ─", "─"]
    );

    // Apparently the "☎" has a width of one?!?
    // I guess thats why its rendered on top of the next character in my terminal...
    assert_eq!("☎".width(), 1);
    assert_eq!(
        WidthChunks::new("_ _ _ _ ☎ shows a telephone", 8).collect::<Vec<_>>(),
        ["_ _ _ _ ", "☎ shows ", "a teleph", "one"]
    );

    assert_eq!(
        WidthChunks::new("#_#_#_#__#_#_#_##_#_#_#__#_#_#_#", 8).collect::<Vec<_>>(),
        ["#_#_#_#_", "_#_#_#_#", "#_#_#_#_", "_#_#_#_#"]
    );
}

#[test]
fn test_rendering() {
    use crate::tree::Element;

    struct TestBackend;
    impl Render for TestBackend {}

    let element = Element::vstack([
        Element::inline("test1"),
        Element::hstack([
            Element::inline("1"),
            Element::inline(" "),
            Element::inline("2"),
            Element::box_(
                [Element::inline("#_#_#_#__#_#_#_##_#_#_#__#_#_#_#")],
                Some(8),
            ),
        ]),
        Element::inline("test3"),
    ]);
    assert_eq!(compute_size(&element), (11, 6));

    let mut writer = Vec::new();
    TestBackend::render(&mut writer, &element).unwrap();
    let output = String::from_utf8(writer).unwrap();

    // Expected:
    // test1
    // 1 2#_#_#_#_
    //    _#_#_#_#
    //    #_#_#_#_
    //    _#_#_#_#
    // test3
    insta::assert_snapshot!(output);
}
