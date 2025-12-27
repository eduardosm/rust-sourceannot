use alloc::format;
use alloc::string::String;

use super::SourceSnippetBuilder;
use crate::SourceSnippet;

impl SourceSnippet {
    /// Creates a snippet from a UTF-8 (possibly broken) source.
    ///
    /// "\n" and "\r\n" are treated as line breaks.
    ///
    /// Control characters (except tabs and line breaks) are represented as
    /// `<XXXX>` as alternative text. Each byte of invalid UTF-8 sequences is
    /// represented as `<XX>` as alternative text.
    pub fn build_from_utf8(start_line: usize, source: &[u8], tab_width: usize) -> Self {
        Self::build_from_utf8_ex(
            start_line,
            source,
            |chr| {
                if chr == '\t' {
                    (false, " ".repeat(tab_width))
                } else {
                    (true, format!("<{:04X}>", u32::from(chr)))
                }
            },
            |bytes| {
                let &[byte] = bytes else {
                    unreachable!();
                };
                (true, format!("<{byte:02X}>"))
            },
            true,
        )
    }

    /// Creates a snippet from a UTF-8 (possibly broken) source.
    ///
    /// "\n" and "\r\n" are treated as line breaks.
    ///
    /// `on_control` is used to handle ASCII control characters (that are not
    /// line breaks). `on_invalid` and `invalid_multi` are used to handle
    /// invalid UTF-8 sequences.
    ///
    /// When `invalid_multi` is `true`, `on_invalid` is called for each byte
    /// of an invalid UTF-8 sequence. Otherwise, `on_invalid` is called once
    /// with the entire sequence.
    ///
    /// `on_control` and `on_invalid` also return a boolean to indicate if the
    /// text should be rendered as alternative.
    pub fn build_from_utf8_ex<FnCtrl, FnInv>(
        start_line: usize,
        source: &[u8],
        mut on_control: FnCtrl,
        mut on_invalid: FnInv,
        invalid_multi: bool,
    ) -> Self
    where
        FnCtrl: FnMut(char) -> (bool, String),
        FnInv: FnMut(&[u8]) -> (bool, String),
    {
        let mut snippet = SourceSnippetBuilder::new(start_line);

        for source_chunk in source.utf8_chunks() {
            let mut chars = source_chunk.valid().chars();
            while let Some(chr) = chars.next() {
                if chr == '\r' && chars.as_str().starts_with('\n') {
                    snippet.next_line(2);
                    chars.next().unwrap();
                } else if chr == '\n' {
                    snippet.next_line(1);
                } else {
                    let chr_width =
                        unicode_width::UnicodeWidthChar::width(chr).filter(|_| chr != '\0');

                    if let Some(chr_width) = chr_width {
                        snippet.push_char(chr, chr_width, chr.len_utf8(), false);
                    } else {
                        let (alt, text) = on_control(chr);
                        snippet.push_text(&text, chr.len_utf8(), alt);
                    }
                }
            }

            let invalid_utf8 = source_chunk.invalid();
            if !invalid_utf8.is_empty() {
                if invalid_multi {
                    for &byte in invalid_utf8.iter() {
                        let (alt, text) = on_invalid(&[byte]);
                        snippet.push_text(&text, 1, alt);
                    }
                } else {
                    let (alt, text) = on_invalid(invalid_utf8);
                    snippet.push_text(&text, invalid_utf8.len(), alt);
                }
            }
        }

        snippet.finish()
    }
}

#[cfg(test)]
mod tests {
    use alloc::format;
    use alloc::string::String;

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
        let snippet = SourceSnippet::build_from_utf8_ex(
            0,
            source,
            |_| unreachable!(),
            |_| unreachable!(),
            false,
        );

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
        let snippet = SourceSnippet::build_from_utf8_ex(
            0,
            source,
            |_| unreachable!(),
            |_| unreachable!(),
            false,
        );

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
        let source = b"123\n4\xC3\xBF6";
        let snippet = SourceSnippet::build_from_utf8_ex(
            0,
            source,
            |_| unreachable!(),
            |_| unreachable!(),
            false,
        );

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
                meta_extra(),
                meta(1, 1),
            ],
        );
    }

    #[test]
    fn test_control_chr() {
        let source = b"123\n4\x006";
        let snippet = SourceSnippet::build_from_utf8(0, source, 4);

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
                    text: "4<0000>6".into(),
                    alts: RangeSet::from(1..=6),
                    width: 8,
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
                meta(6, 6),
                meta(1, 1),
            ],
        );
    }

    #[test]
    fn test_crlf() {
        let source = b"123\r\n4\r6\r\n";
        let snippet = SourceSnippet::build_from_utf8_ex(
            0,
            source,
            |chr| (true, format!("<{:02X}>", chr as u8)),
            |_| unreachable!(),
            false,
        );

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
    fn test_fullwidth() {
        let source = b"1\xEF\xBC\x923\n456";
        let snippet = SourceSnippet::build_from_utf8_ex(
            0,
            source,
            |_| unreachable!(),
            |_| unreachable!(),
            false,
        );

        assert_eq!(snippet.start_line, 0);
        assert_eq!(
            snippet.lines,
            [
                SourceLine {
                    text: "1\u{FF12}3".into(),
                    alts: RangeSet::new(),
                    width: 4,
                },
                SourceLine {
                    text: "456".into(),
                    alts: RangeSet::new(),
                    width: 3,
                },
            ],
        );
        assert_eq!(snippet.line_map, [6]);
        assert_eq!(
            snippet.metas,
            [
                meta(1, 1),
                meta(2, 3),
                meta_extra(),
                meta_extra(),
                meta(1, 1),
                meta(1, 0),
                meta(1, 1),
                meta(1, 1),
                meta(1, 1),
            ],
        );
    }

    #[test]
    fn test_tabs() {
        let source = b"123\n\t456";
        let snippet = SourceSnippet::build_from_utf8(0, source, 4);

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

    #[test]
    fn test_invalid_single() {
        let source = b"1\xF1\x803\n456";
        let snippet = SourceSnippet::build_from_utf8_ex(
            0,
            source,
            |_| unreachable!(),
            |bytes| {
                let mut s = String::new();
                s.push('<');
                for &byte in bytes {
                    s.push_str(&format!("{byte:02X}"));
                }
                s.push('>');
                (true, s)
            },
            false,
        );

        assert_eq!(snippet.start_line, 0);
        assert_eq!(
            snippet.lines,
            [
                SourceLine {
                    text: "1<F180>3".into(),
                    alts: RangeSet::from(1..=6),
                    width: 8,
                },
                SourceLine {
                    text: "456".into(),
                    alts: RangeSet::new(),
                    width: 3,
                },
            ],
        );
        assert_eq!(snippet.line_map, [5]);
        assert_eq!(
            snippet.metas,
            [
                meta(1, 1),
                meta(6, 6),
                meta_extra(),
                meta(1, 1),
                meta(1, 0),
                meta(1, 1),
                meta(1, 1),
                meta(1, 1),
            ],
        );
    }

    #[test]
    fn test_invalid_multi() {
        let source = b"1\xF1\x803\n456";
        let snippet = SourceSnippet::build_from_utf8_ex(
            0,
            source,
            |_| unreachable!(),
            |bytes| {
                assert_eq!(bytes.len(), 1);
                let s = format!("<{:02X}>", bytes[0]);
                (true, s)
            },
            true,
        );

        assert_eq!(snippet.start_line, 0);
        assert_eq!(
            snippet.lines,
            [
                SourceLine {
                    text: "1<F1><80>3".into(),
                    alts: RangeSet::from(1..=8),
                    width: 10,
                },
                SourceLine {
                    text: "456".into(),
                    alts: RangeSet::new(),
                    width: 3,
                },
            ],
        );
        assert_eq!(snippet.line_map, [5]);
        assert_eq!(
            snippet.metas,
            [
                meta(1, 1),
                meta(4, 4),
                meta(4, 4),
                meta(1, 1),
                meta(1, 0),
                meta(1, 1),
                meta(1, 1),
                meta(1, 1),
            ],
        );
    }
}
