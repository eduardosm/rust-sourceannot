use super::{SourceLine, SourceSnippet, SourceUnitMeta};
use crate::range_set::RangeSet;

struct SourceSnippetBuilder {
    start_line: usize,
    lines: Vec<SourceLine>,
    line_map: Vec<usize>,
    metas: Vec<SourceUnitMeta>,
    current_line_text: String,
    current_line_alts: RangeSet<usize>,
    current_line_width: usize,
}

impl SourceSnippetBuilder {
    fn new(start_line: usize) -> Self {
        Self {
            start_line,
            lines: Vec::new(),
            line_map: Vec::new(),
            metas: Vec::new(),
            current_line_text: String::new(),
            current_line_alts: RangeSet::new(),
            current_line_width: 0,
        }
    }

    fn finish(mut self) -> SourceSnippet {
        self.lines.push(SourceLine {
            text: self.current_line_text.into_boxed_str(),
            alts: self.current_line_alts,
            width: self.current_line_width,
        });

        SourceSnippet {
            start_line: self.start_line,
            lines: self.lines,
            line_map: self.line_map,
            metas: self.metas,
        }
    }

    fn next_line(&mut self, extra_widths: &[usize]) {
        self.lines.push(SourceLine {
            text: std::mem::take(&mut self.current_line_text).into_boxed_str(),
            alts: std::mem::take(&mut self.current_line_alts),
            width: std::mem::replace(&mut self.current_line_width, 0),
        });
        self.metas
            .extend(extra_widths.iter().map(|&w| SourceUnitMeta::new(w, 0)));
        self.line_map.push(self.metas.len());
    }

    fn push_text(&mut self, text: &str, orig_len: usize, alt: bool) {
        let old_line_len = self.current_line_text.len();
        self.current_line_text.push_str(text);
        let new_line_len = self.current_line_text.len();

        if alt && !text.is_empty() {
            self.current_line_alts
                .insert(old_line_len..=(new_line_len - 1));
        }

        let width = unicode_width::UnicodeWidthStr::width(text);
        self.current_line_width += width;

        self.metas.push(SourceUnitMeta::new(width, text.len()));
        for _ in 1..orig_len {
            // Each element of `snippet.widths` corresponds to a byte in `source`,
            // so fill with -1 for multi-unit chunks.
            self.metas.push(SourceUnitMeta::extra());
        }
    }

    fn push_char(&mut self, chr: char, width: usize, orig_len: usize, alt: bool) {
        let old_line_len = self.current_line_text.len();
        self.current_line_text.push(chr);
        let new_line_len = self.current_line_text.len();
        self.current_line_width += width;

        if alt {
            self.current_line_alts
                .insert(old_line_len..=(new_line_len - 1));
        }

        self.metas.push(SourceUnitMeta::new(width, chr.len_utf8()));
        for _ in 1..orig_len {
            // Each element of `snippet.widths` corresponds to a byte in `source`,
            // so fill with -1 for multi-unit characters.
            self.metas.push(SourceUnitMeta::extra());
        }
    }
}

impl SourceSnippet {
    /// Creates a snippet from a UTF-8 (possibly broken) source.
    ///
    /// "\n" and "\r\n" are treated as line breaks.
    ///
    /// Each byte of ASCII control characters (that are not line breaks) and
    /// invalid UTF-8 sequences are represented as `<XX>` as alternative text.
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
    /// `on_control` and `on_invalid` also returns a boolean to indicate if
    /// the text should be rendered as alternative.
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

        let mut rem_source = source;
        while !rem_source.is_empty() {
            let valid_utf8;
            let invalid_utf8: &[u8];
            match std::str::from_utf8(rem_source) {
                Ok(s) => {
                    valid_utf8 = s;
                    invalid_utf8 = b"";
                    rem_source = b"";
                }
                Err(e) => {
                    let (valid, after_valid) = rem_source.split_at(e.valid_up_to());
                    let error_len = e.error_len().unwrap_or(after_valid.len());
                    valid_utf8 = std::str::from_utf8(valid).unwrap();
                    (invalid_utf8, rem_source) = after_valid.split_at(error_len);
                }
            }

            let mut chars = valid_utf8.chars();
            while let Some(chr) = chars.next() {
                if chr == '\r' && chars.as_str().starts_with('\n') {
                    snippet.next_line(&[1, 1]);
                    chars.next().unwrap();
                } else if chr == '\n' {
                    snippet.next_line(&[1]);
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
    use super::{RangeSet, SourceLine, SourceSnippet, SourceUnitMeta};

    fn meta(width: usize, len: usize) -> SourceUnitMeta {
        SourceUnitMeta::new(width, len)
    }

    fn meta_extra() -> SourceUnitMeta {
        SourceUnitMeta::extra()
    }

    #[test]
    fn test_utf8_simple_1() {
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
    fn test_utf8_simple_2() {
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
    fn test_utf8_crlf() {
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
                meta(1, 0),
                meta(1, 1),
                meta(4, 4),
                meta(1, 1),
                meta(1, 0),
                meta(1, 0),
            ],
        );
    }

    #[test]
    fn test_utf8_fullwidth() {
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
    fn test_utf8_tabs() {
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
    fn test_utf8_invalid_single() {
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
    fn test_utf8_invalid_multi() {
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
