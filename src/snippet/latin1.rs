use super::{ControlCharStyle, Snippet};

impl Snippet {
    /// Creates a snippet from a Latin-1 (ISO 8859-1) source.
    ///
    /// "\n" and "\r\n" are treated as line breaks.
    pub fn build_from_latin1(
        start_line: usize,
        source: &[u8],
        tab_width: usize,
        control_char_style: ControlCharStyle,
        control_char_alt: bool,
    ) -> Self {
        let mut builder = Snippet::builder(start_line);

        let mut chars = source.iter();
        while let Some(&chr) = chars.next() {
            if chr == b'\r' && chars.as_slice().starts_with(b"\n") {
                builder.next_line(2);
                chars.next().unwrap();
            } else if chr == b'\n' {
                builder.next_line(1);
            } else {
                let is_control = builder.maybe_push_control_char(
                    chr.into(),
                    1,
                    tab_width,
                    control_char_style,
                    control_char_alt,
                );
                if !is_control {
                    builder.push_char(chr.into(), 1, false);
                }
            }
        }

        builder.finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::range_set::RangeSet;
    use crate::snippet::{ControlCharStyle, Snippet, SnippetLine, UnitMeta};

    fn meta(width: u8, len: u8) -> UnitMeta {
        UnitMeta::new(width, len)
    }

    fn meta_extra() -> UnitMeta {
        UnitMeta::extra()
    }

    #[test]
    fn test_simple_1() {
        let source = b"123\n456";
        let snippet = Snippet::build_from_latin1(0, source, 4, ControlCharStyle::Hexadecimal, true);

        assert_eq!(snippet.start_line, 0);
        assert_eq!(
            snippet.lines,
            [
                SnippetLine {
                    text: "123".into(),
                    alts: RangeSet::new(),
                },
                SnippetLine {
                    text: "456".into(),
                    alts: RangeSet::new(),
                },
            ],
        );
        assert_eq!(snippet.line_map, [4]);
        assert_eq!(
            snippet.metas,
            [
                meta(1, 1),
                meta(1, 1),
                meta(1, 1),
                meta(1, 0),
                meta(1, 1),
                meta(1, 1),
                meta(1, 1),
            ],
        );
    }

    #[test]
    fn test_simple_2() {
        let source = b"123\n456\n";
        let snippet = Snippet::build_from_latin1(0, source, 4, ControlCharStyle::Hexadecimal, true);

        assert_eq!(snippet.start_line, 0);
        assert_eq!(
            snippet.lines,
            [
                SnippetLine {
                    text: "123".into(),
                    alts: RangeSet::new(),
                },
                SnippetLine {
                    text: "456".into(),
                    alts: RangeSet::new(),
                },
                SnippetLine {
                    text: "".into(),
                    alts: RangeSet::new(),
                },
            ],
        );
        assert_eq!(snippet.line_map, [4, 8]);
        assert_eq!(
            snippet.metas,
            [
                meta(1, 1),
                meta(1, 1),
                meta(1, 1),
                meta(1, 0),
                meta(1, 1),
                meta(1, 1),
                meta(1, 1),
                meta(1, 0),
            ],
        );
    }

    #[test]
    fn test_non_ascii_chr() {
        let source = b"123\n4\xFF6";
        let snippet = Snippet::build_from_latin1(0, source, 4, ControlCharStyle::Hexadecimal, true);

        assert_eq!(snippet.start_line, 0);
        assert_eq!(
            snippet.lines,
            [
                SnippetLine {
                    text: "123".into(),
                    alts: RangeSet::new(),
                },
                SnippetLine {
                    text: "4\u{FF}6".into(),
                    alts: RangeSet::new(),
                },
            ],
        );
        assert_eq!(snippet.line_map, [4]);
        assert_eq!(
            snippet.metas,
            [
                meta(1, 1),
                meta(1, 1),
                meta(1, 1),
                meta(1, 0),
                meta(1, 1),
                meta(1, 2),
                meta(1, 1),
            ],
        );
    }

    #[test]
    fn test_control_chr() {
        let source = b"123\n4\x806";
        let snippet = Snippet::build_from_latin1(0, source, 4, ControlCharStyle::Hexadecimal, true);

        assert_eq!(snippet.start_line, 0);
        assert_eq!(
            snippet.lines,
            [
                SnippetLine {
                    text: "123".into(),
                    alts: RangeSet::new(),
                },
                SnippetLine {
                    text: "4<80>6".into(),
                    alts: RangeSet::from(1..=4),
                },
            ],
        );
        assert_eq!(snippet.line_map, [4]);
        assert_eq!(
            snippet.metas,
            [
                meta(1, 1),
                meta(1, 1),
                meta(1, 1),
                meta(1, 0),
                meta(1, 1),
                meta(4, 4),
                meta(1, 1),
            ],
        );
    }

    #[test]
    fn test_crlf() {
        let source = b"123\r\n4\r6\r\n";
        let snippet = Snippet::build_from_latin1(0, source, 4, ControlCharStyle::Hexadecimal, true);

        assert_eq!(snippet.start_line, 0);
        assert_eq!(
            snippet.lines,
            [
                SnippetLine {
                    text: "123".into(),
                    alts: RangeSet::new(),
                },
                SnippetLine {
                    text: "4<0D>6".into(),
                    alts: RangeSet::from(1..=4),
                },
                SnippetLine {
                    text: "".into(),
                    alts: RangeSet::new(),
                },
            ],
        );
        assert_eq!(snippet.line_map, [5, 10]);
        assert_eq!(
            snippet.metas,
            [
                meta(1, 1),
                meta(1, 1),
                meta(1, 1),
                meta(1, 0),
                meta_extra(),
                meta(1, 1),
                meta(4, 4),
                meta(1, 1),
                meta(1, 0),
                meta_extra(),
            ],
        );
    }

    #[test]
    fn test_tabs() {
        let source = b"123\n\t456";
        let snippet = Snippet::build_from_latin1(0, source, 4, ControlCharStyle::Hexadecimal, true);

        assert_eq!(snippet.start_line, 0);
        assert_eq!(
            snippet.lines,
            [
                SnippetLine {
                    text: "123".into(),
                    alts: RangeSet::new(),
                },
                SnippetLine {
                    text: "    456".into(),
                    alts: RangeSet::new(),
                },
            ],
        );
        assert_eq!(snippet.line_map, [4]);
        assert_eq!(
            snippet.metas,
            [
                meta(1, 1),
                meta(1, 1),
                meta(1, 1),
                meta(1, 0),
                meta(4, 4),
                meta(1, 1),
                meta(1, 1),
                meta(1, 1),
            ],
        );
    }
}
