use alloc::format;
use alloc::string::String;

use super::Snippet;

impl Snippet {
    /// Creates a snippet from a Latin-1 (ISO 8859-1) source.
    ///
    /// "\n" and "\r\n" are treated as line breaks.
    ///
    /// Control characters (except tabs and line breaks) are represented as
    /// `<XX>` as alternative text.
    pub fn build_from_latin1(start_line: usize, source: &[u8], tab_width: usize) -> Self {
        Self::build_from_latin1_ex(start_line, source, |chr| {
            if chr == b'\t' {
                (false, " ".repeat(tab_width))
            } else {
                (true, format!("<{chr:02X}>"))
            }
        })
    }

    /// Creates a snippet from a Latin-1 (ISO 8859-1) source.
    ///
    /// "\n" and "\r\n" are treated as line breaks.
    ///
    /// `on_control` is used to handle control characters (that are not line
    /// breaks). `on_control` also returns a boolean to indicate if the text
    /// should be rendered as alternative.
    ///
    /// `on_control` should not return a string that contains tabs, line breaks
    /// or any other control characters.
    pub fn build_from_latin1_ex<FnCtrl>(
        start_line: usize,
        source: &[u8],
        mut on_control: FnCtrl,
    ) -> Self
    where
        FnCtrl: FnMut(u8) -> (bool, String),
    {
        let mut snippet = Snippet::builder(start_line);

        let mut chars = source.iter();
        while let Some(&chr) = chars.next() {
            if chr == b'\r' && chars.as_slice().starts_with(b"\n") {
                snippet.next_line(2);
                chars.next().unwrap();
            } else if chr == b'\n' {
                snippet.next_line(1);
            } else if matches!(chr, b' '..=b'~' | 0xA0..=0xFF) {
                snippet.push_char(chr.into(), 1, false);
            } else {
                let (alt, text) = on_control(chr);
                snippet.push_str(&text, 1, alt);
            }
        }

        snippet.finish()
    }
}

#[cfg(test)]
mod tests {
    use alloc::format;

    use crate::range_set::RangeSet;
    use crate::snippet::{Snippet, SnippetLine, UnitMeta};

    fn meta(width: u8, len: u8) -> UnitMeta {
        UnitMeta::new(width, len)
    }

    fn meta_extra() -> UnitMeta {
        UnitMeta::extra()
    }

    #[test]
    fn test_simple_1() {
        let source = b"123\n456";
        let snippet = Snippet::build_from_latin1_ex(0, source, |_| unreachable!());

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
        let snippet = Snippet::build_from_latin1_ex(0, source, |_| unreachable!());

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
        let snippet = Snippet::build_from_latin1_ex(0, source, |_| unreachable!());

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
        let snippet = Snippet::build_from_latin1(0, source, 4);

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
        let snippet =
            Snippet::build_from_latin1_ex(0, source, |chr| (true, format!("<{chr:02X}>")));

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
        let snippet = Snippet::build_from_latin1(0, source, 4);

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

    #[test]
    fn test_large_meta() {
        let source = b"1\x002";
        let snippet = Snippet::build_from_latin1_ex(0, source, |_| (true, "\u{A7}".repeat(150)));

        assert_eq!(snippet.start_line, 0);
        assert_eq!(
            snippet.lines,
            [SnippetLine {
                text: format!("1{}2", "\u{A7}".repeat(150)).into_boxed_str(),
                alts: RangeSet::from(1..=300),
            }],
        );
        assert_eq!(snippet.line_map, []);
        assert_eq!(snippet.metas, [meta(1, 1), meta(0x7F, 0x7F), meta(1, 1)]);
        assert_eq!(snippet.large_widths, [(1, 150)]);
        assert_eq!(snippet.large_utf8_lens, [(1, 300)]);
    }
}
