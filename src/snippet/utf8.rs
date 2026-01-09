use super::{ControlCharStyle, Snippet};

/// Style for how invalid UTF-8 sequences are represented.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum InvalidUtf8SeqStyle {
    /// Represent invalid UTF-8 as the replacement character U+FFFD (`�`).
    Replacement,
    /// Represent invalid UTF-8 as hexadecimal byte values, each byte as `<XX>`.
    ///
    /// This is useful when you want the invalid bytes to be visible and
    /// "countable" in the rendered output.
    Hexadecimal,
}

impl Snippet {
    /// Creates a [`Snippet`] from a UTF-8 (possibly invalid) source.
    ///
    /// # Source units and spans
    ///
    /// The *source unit* for this builder is a **byte** of the original `source`.
    /// Any annotation span you pass later (a `Range<usize>`) is interpreted as
    /// byte offsets into this original `source` slice.
    ///
    /// # Line breaks
    ///
    /// - `\n` and `\r\n` are treated as line breaks.
    /// - A lone `\r` is *not* a line break; it is handled like any other control
    ///   character.
    ///
    /// # Control characters
    ///
    /// Tabs (`\t`) are expanded to `tab_width` spaces. Other control characters
    /// are rendered according to `control_char_style` (see [`ControlCharStyle`]).
    /// If `control_char_alt` is `true`, those replacement fragments are marked as
    /// "alternate" text.
    ///
    /// # Invalid UTF-8
    ///
    /// When malformed UTF-8 is encountered, it is rendered according to
    /// `invalid_seq_style` (see [`InvalidUtf8SeqStyle`]). If `invalid_seq_alt` is
    /// `true`, the replacement fragments are marked as "alternate" text.
    pub fn build_from_utf8(
        start_line: usize,
        source: &[u8],
        tab_width: usize,
        control_char_style: ControlCharStyle,
        control_char_alt: bool,
        invalid_seq_style: InvalidUtf8SeqStyle,
        invalid_seq_alt: bool,
    ) -> Self {
        let mut snippet = Snippet::builder(start_line);

        for source_chunk in source.utf8_chunks() {
            let mut chars = source_chunk.valid().chars();
            while let Some(chr) = chars.next() {
                if chr == '\r' && chars.as_str().starts_with('\n') {
                    snippet.next_line(2);
                    chars.next().unwrap();
                } else if chr == '\n' {
                    snippet.next_line(1);
                } else {
                    let chr_len = chr.len_utf8();
                    let is_control = snippet.maybe_push_control_char(
                        chr,
                        chr_len,
                        tab_width,
                        control_char_style,
                        control_char_alt,
                    );
                    if !is_control {
                        snippet.push_char(chr, chr_len, false);
                    }
                }
            }

            let invalid_utf8 = source_chunk.invalid();
            if !invalid_utf8.is_empty() {
                match invalid_seq_style {
                    InvalidUtf8SeqStyle::Replacement => {
                        snippet.push_char('\u{FFFD}', invalid_utf8.len(), invalid_seq_alt);
                    }
                    InvalidUtf8SeqStyle::Hexadecimal => {
                        for &byte in invalid_utf8.iter() {
                            snippet.push_fmt(format_args!("<{byte:02X}>"), 1, invalid_seq_alt);
                        }
                    }
                }
            }
        }

        snippet.finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::range_set::RangeSet;
    use crate::snippet::{ControlCharStyle, InvalidUtf8SeqStyle, Snippet, SnippetLine, UnitMeta};

    fn meta(width: u8, len: u8) -> UnitMeta {
        UnitMeta::new(width, len)
    }

    fn meta_extra() -> UnitMeta {
        UnitMeta::extra()
    }

    #[test]
    fn test_simple_1() {
        let source = b"123\n456";
        let snippet = Snippet::build_from_utf8(
            0,
            source,
            4,
            ControlCharStyle::Hexadecimal,
            true,
            InvalidUtf8SeqStyle::Hexadecimal,
            true,
        );

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
        let snippet = Snippet::build_from_utf8(
            0,
            source,
            4,
            ControlCharStyle::Hexadecimal,
            true,
            InvalidUtf8SeqStyle::Hexadecimal,
            true,
        );

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
        let source = b"123\n4\xC3\xBF6";
        let snippet = Snippet::build_from_utf8(
            0,
            source,
            4,
            ControlCharStyle::Hexadecimal,
            true,
            InvalidUtf8SeqStyle::Hexadecimal,
            true,
        );

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
                meta_extra(),
                meta(1, 1),
            ],
        );
    }

    #[test]
    fn test_control_chr() {
        let source = b"123\n4\x006";
        let snippet = Snippet::build_from_utf8(
            0,
            source,
            4,
            ControlCharStyle::Hexadecimal,
            true,
            InvalidUtf8SeqStyle::Hexadecimal,
            true,
        );

        assert_eq!(snippet.start_line, 0);
        assert_eq!(
            snippet.lines,
            [
                SnippetLine {
                    text: "123".into(),
                    alts: RangeSet::new(),
                },
                SnippetLine {
                    text: "4<00>6".into(),
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
        let snippet = Snippet::build_from_utf8(
            0,
            source,
            4,
            ControlCharStyle::Hexadecimal,
            true,
            InvalidUtf8SeqStyle::Hexadecimal,
            true,
        );

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
    fn test_fullwidth() {
        let source = b"1\xEF\xBC\x923\n456";
        let snippet = Snippet::build_from_utf8(
            0,
            source,
            4,
            ControlCharStyle::Hexadecimal,
            true,
            InvalidUtf8SeqStyle::Hexadecimal,
            true,
        );

        assert_eq!(snippet.start_line, 0);
        assert_eq!(
            snippet.lines,
            [
                SnippetLine {
                    text: "1\u{FF12}3".into(),
                    alts: RangeSet::new(),
                },
                SnippetLine {
                    text: "456".into(),
                    alts: RangeSet::new(),
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
        let snippet = Snippet::build_from_utf8(
            0,
            source,
            4,
            ControlCharStyle::Hexadecimal,
            true,
            InvalidUtf8SeqStyle::Hexadecimal,
            true,
        );

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
    fn test_invalid() {
        let source = b"1\xF1\x803\n456";
        let snippet = Snippet::build_from_utf8(
            0,
            source,
            4,
            ControlCharStyle::Hexadecimal,
            true,
            InvalidUtf8SeqStyle::Hexadecimal,
            true,
        );

        assert_eq!(snippet.start_line, 0);
        assert_eq!(
            snippet.lines,
            [
                SnippetLine {
                    text: "1<F1><80>3".into(),
                    alts: RangeSet::from(1..=8),
                },
                SnippetLine {
                    text: "456".into(),
                    alts: RangeSet::new(),
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
