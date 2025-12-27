use alloc::format;
use alloc::string::String;

use super::SourceSnippetBuilder;
use crate::SourceSnippet;

impl SourceSnippet {
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
    pub fn build_from_latin1_ex<FnCtrl>(
        start_line: usize,
        source: &[u8],
        mut on_control: FnCtrl,
    ) -> Self
    where
        FnCtrl: FnMut(u8) -> (bool, String),
    {
        let mut snippet = SourceSnippetBuilder::new(start_line);

        let mut chars = source.iter();
        while let Some(&chr) = chars.next() {
            if chr == b'\r' && chars.as_slice().starts_with(b"\n") {
                snippet.next_line(2);
                chars.next().unwrap();
            } else if chr == b'\n' {
                snippet.next_line(1);
            } else {
                let orig_len = 1;

                if matches!(chr, b' '..=b'~' | 0xA0..=0xFF) {
                    // The width of all printable Latin-1 characters is 1.
                    let chr_width = 1;
                    snippet.push_char(chr.into(), chr_width, orig_len, false);
                } else {
                    let (alt, text) = on_control(chr);
                    snippet.push_text(&text, orig_len, alt);
                }
            }
        }

        snippet.finish()
    }
}

#[cfg(test)]
mod tests {
    use alloc::format;

    use crate::range_set::RangeSet;
    use crate::snippet::{SourceLine, SourceSnippet, SourceUnitMeta};

    fn meta(width: usize, len: usize) -> SourceUnitMeta {
        SourceUnitMeta::new(width, len)
    }

    fn meta_extra() -> SourceUnitMeta {
        SourceUnitMeta::extra()
    }

    #[test]
    fn test_simple_1() {
        let source = b"123\n456";
        let snippet = SourceSnippet::build_from_latin1_ex(0, source, |_| unreachable!());

        assert_eq!(snippet.start_line, 0);
        assert_eq!(snippet.lines.len(), 2);
        assert_eq!(
            snippet.lines,
            [
                SourceLine {
                    text: "123".into(),
                    alts: RangeSet::new(),
                    width: 3,
                },
                SourceLine {
                    text: "456".into(),
                    alts: RangeSet::new(),
                    width: 3,
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
        let snippet = SourceSnippet::build_from_latin1_ex(0, source, |_| unreachable!());

        assert_eq!(snippet.start_line, 0);
        assert_eq!(snippet.lines.len(), 3);
        assert_eq!(
            snippet.lines,
            [
                SourceLine {
                    text: "123".into(),
                    alts: RangeSet::new(),
                    width: 3,
                },
                SourceLine {
                    text: "456".into(),
                    alts: RangeSet::new(),
                    width: 3,
                },
                SourceLine {
                    text: "".into(),
                    alts: RangeSet::new(),
                    width: 0,
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
        let snippet = SourceSnippet::build_from_latin1_ex(0, source, |_| unreachable!());

        assert_eq!(snippet.start_line, 0);
        assert_eq!(snippet.lines.len(), 2);
        assert_eq!(
            snippet.lines,
            [
                SourceLine {
                    text: "123".into(),
                    alts: RangeSet::new(),
                    width: 3,
                },
                SourceLine {
                    text: "4\u{FF}6".into(),
                    alts: RangeSet::new(),
                    width: 3,
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
        let snippet = SourceSnippet::build_from_latin1(0, source, 4);

        assert_eq!(snippet.start_line, 0);
        assert_eq!(snippet.lines.len(), 2);
        assert_eq!(
            snippet.lines,
            [
                SourceLine {
                    text: "123".into(),
                    alts: RangeSet::new(),
                    width: 3,
                },
                SourceLine {
                    text: "4<80>6".into(),
                    alts: RangeSet::from(1..=4),
                    width: 6,
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
            SourceSnippet::build_from_latin1_ex(0, source, |chr| (true, format!("<{chr:02X}>")));

        assert_eq!(snippet.start_line, 0);
        assert_eq!(snippet.lines.len(), 3);
        assert_eq!(
            snippet.lines,
            [
                SourceLine {
                    text: "123".into(),
                    alts: RangeSet::new(),
                    width: 3,
                },
                SourceLine {
                    text: "4<0D>6".into(),
                    alts: RangeSet::from(1..=4),
                    width: 6,
                },
                SourceLine {
                    text: "".into(),
                    alts: RangeSet::new(),
                    width: 0,
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
        let snippet = SourceSnippet::build_from_latin1(0, source, 4);

        assert_eq!(snippet.start_line, 0);
        assert_eq!(snippet.lines.len(), 2);
        assert_eq!(
            snippet.lines,
            [
                SourceLine {
                    text: "123".into(),
                    alts: RangeSet::new(),
                    width: 3,
                },
                SourceLine {
                    text: "    456".into(),
                    alts: RangeSet::new(),
                    width: 7,
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
