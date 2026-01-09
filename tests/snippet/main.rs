mod latin1;
mod utf8;

const MAIN_STYLE: sourceannot::MainStyle<char> = sourceannot::MainStyle {
    margin: None,
    horizontal_char: '─',
    vertical_char: '│',
    top_vertical_char: '╭',
    top_corner_char: '╭',
    bottom_corner_char: '╰',
    spaces_meta: 's',
    text_normal_meta: 't',
    text_alt_meta: 'T',
};

const ANNOT_STYLE: sourceannot::AnnotStyle<char> = sourceannot::AnnotStyle {
    caret: '^',
    text_normal_meta: 't',
    text_alt_meta: 'T',
    line_meta: 'l',
};

struct TestOutput {
    text: String,
    styles: String,
}

impl sourceannot::Output<char> for &mut TestOutput {
    type Error = core::convert::Infallible;

    fn put_str(&mut self, s: &str, &style: &char) -> Result<(), Self::Error> {
        self.text.push_str(s);
        for c in s.chars() {
            self.styles.push(style);
            if c == '\n' {
                self.styles.push('\n');
            }
        }

        Ok(())
    }
}

#[track_caller]
fn test_render_simple(
    snippet: &dyn sourceannot::Snippet,
    span: std::ops::Range<usize>,
    text_line: &str,
    carets_line: &str,
    text_styles: &str,
) {
    let mut annots = sourceannot::Annotations::new(snippet, 1, MAIN_STYLE);
    annots.add_annotation(span, ANNOT_STYLE, Vec::new());

    let mut output = TestOutput {
        text: String::new(),
        styles: String::new(),
    };
    let max_line_no_width = annots.max_line_no_width();
    annots.render(max_line_no_width, 0, 0, &mut output);

    let carets_line = carets_line.trim_end_matches(' ');
    let mut expected_styles = String::new();
    expected_styles.push_str(text_styles);
    expected_styles.push_str("s\n");
    for c in carets_line.chars() {
        match c {
            ' ' => expected_styles.push('s'),
            '^' => expected_styles.push('l'),
            _ => unreachable!(),
        }
    }
    expected_styles.push_str("s\n");

    assert_eq!(output.text, format!("{text_line}\n{carets_line}\n"));
    assert_eq!(output.styles, expected_styles);
}

#[test]
fn test_large_chunk() {
    struct TestSnippet {
        n: usize,
        line_map: sourceannot::LineMap,
    }

    impl TestSnippet {
        fn new(n: usize) -> Self {
            Self {
                n,
                line_map: sourceannot::LineMap::builder(1 + n + 1).finish(),
            }
        }
    }

    impl sourceannot::Snippet for TestSnippet {
        fn line_map(&self) -> &sourceannot::LineMap {
            &self.line_map
        }

        fn get_line(&self, line_i: usize) -> sourceannot::SnippetLine {
            assert_eq!(line_i, 0);

            let mut builder = sourceannot::SnippetLine::builder();
            builder.push_char('a', 1, false);
            builder.push_str(&"¬".repeat(self.n), 4, true);
            builder.push_char('b', 1, false);
            builder.finish(0)
        }
    }

    for n in 0..=500 {
        let snippet = TestSnippet::new(n);

        let rendered_line = format!("a{}b", "¬".repeat(n));
        let rendered_line_style = format!("t{}t", "T".repeat(n));
        test_render_simple(
            &snippet,
            0..1,
            &rendered_line,
            &format!("^{} ", " ".repeat(n)),
            &rendered_line_style,
        );
        test_render_simple(
            &snippet,
            1..2,
            &rendered_line,
            &format!(" {} ", "^".repeat(n.max(1))),
            &rendered_line_style,
        );
        test_render_simple(
            &snippet,
            2..3,
            &rendered_line,
            &format!(" {} ", "^".repeat(n.max(1))),
            &rendered_line_style,
        );
        test_render_simple(
            &snippet,
            3..4,
            &rendered_line,
            &format!(" {} ", "^".repeat(n.max(1))),
            &rendered_line_style,
        );
        test_render_simple(
            &snippet,
            4..5,
            &rendered_line,
            &format!(" {} ", "^".repeat(n.max(1))),
            &rendered_line_style,
        );
        test_render_simple(
            &snippet,
            1..5,
            &rendered_line,
            &format!(" {} ", "^".repeat(n.max(1))),
            &rendered_line_style,
        );
        test_render_simple(
            &snippet,
            5..6,
            &rendered_line,
            &format!(" {}^", " ".repeat(n)),
            &rendered_line_style,
        );
    }
}
