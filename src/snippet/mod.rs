use alloc::boxed::Box;
use alloc::vec::Vec;

mod build;

use crate::range_set::RangeSet;

/// A snippet of source code.
#[derive(Clone, Debug)]
pub struct SourceSnippet {
    start_line: usize,
    lines: Vec<SourceLine>,
    line_map: Vec<usize>,
    metas: Vec<SourceUnitMeta>,
    large_widths: Vec<(usize, usize)>,
    large_utf8_lens: Vec<(usize, usize)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SourceLine {
    pub(crate) text: Box<str>,
    pub(crate) alts: RangeSet<usize>,
    width: usize,
}

#[derive(Clone, PartialEq, Eq)]
struct SourceUnitMeta {
    inner: u16,
}

impl core::fmt::Debug for SourceUnitMeta {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.is_extra() {
            f.write_str("SourceUnitMeta::extra()")
        } else {
            f.debug_struct("SourceUnitMeta")
                .field("width", &self.width())
                .field("utf8_len", &self.utf8_len())
                .finish()
        }
    }
}

impl SourceUnitMeta {
    #[inline]
    fn extra() -> Self {
        Self { inner: 0x8000 }
    }

    #[inline]
    fn new(width: u8, utf8_len: u8) -> Self {
        assert!(width <= 0x7F);
        assert!(utf8_len <= 0x7F);
        Self {
            inner: u16::from(width) | (u16::from(utf8_len) << 7),
        }
    }

    #[inline]
    fn is_extra(&self) -> bool {
        self.inner & 0x8000 != 0
    }

    #[inline]
    fn width(&self) -> u8 {
        (self.inner & 0x7F) as u8
    }

    #[inline]
    fn utf8_len(&self) -> u8 {
        ((self.inner >> 7) & 0x7F) as u8
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SourceSpan {
    pub(crate) start_line: usize,
    pub(crate) start_col: usize,
    pub(crate) start_utf8: usize,
    pub(crate) end_line: usize,
    pub(crate) end_col: usize,
    pub(crate) end_utf8: usize,
}

impl SourceSnippet {
    pub fn get_line_col(&self, pos: usize) -> (usize, usize) {
        let line = match self.line_map.binary_search(&pos) {
            Ok(i) => i + 1,
            Err(i) => i,
        };
        let line_start = if line == 0 {
            0
        } else {
            self.line_map[line - 1]
        };
        let col = self.gather_width(line_start..pos);

        (line, col)
    }

    fn gather_utf8_len(&self, range: core::ops::Range<usize>) -> usize {
        let mut utf8_len = 0;
        for (i, meta) in self.metas[range.clone()].iter().enumerate() {
            let len_i = meta.utf8_len();
            if len_i == 0x7F {
                let large_i = self
                    .large_utf8_lens
                    .binary_search_by_key(&(i + range.start), |&(j, _)| j)
                    .unwrap();
                utf8_len += self.large_utf8_lens[large_i].1;
            } else {
                utf8_len += usize::from(len_i);
            }
        }
        utf8_len
    }

    fn gather_width(&self, range: core::ops::Range<usize>) -> usize {
        let mut width = 0;
        for (i, meta) in self.metas[range.clone()].iter().enumerate() {
            let width_i = meta.width();
            if width_i == 0x7F {
                let large_i = self
                    .large_widths
                    .binary_search_by_key(&(i + range.start), |&(j, _)| j)
                    .unwrap();
                width += self.large_widths[large_i].1;
            } else {
                width += usize::from(width_i);
            }
        }
        width
    }

    #[inline]
    pub(crate) fn start_line(&self) -> usize {
        self.start_line
    }

    #[inline]
    pub(crate) fn line(&self, i: usize) -> &SourceLine {
        &self.lines[i]
    }

    pub(crate) fn convert_span(&self, mut start: usize, mut end: usize) -> SourceSpan {
        end = end.max(start);

        while self.metas.get(start).is_some_and(SourceUnitMeta::is_extra) {
            start -= 1;
        }
        while self.metas.get(end).is_some_and(SourceUnitMeta::is_extra) {
            end += 1;
        }
        start = start.min(self.metas.len());
        end = end.min(self.metas.len());

        let start_line = match self.line_map.binary_search(&start) {
            Ok(i) => i + 1,
            Err(i) => i,
        };
        let start_line_start = if start_line == 0 {
            0
        } else {
            self.line_map[start_line - 1]
        };
        let start_col = self.gather_width(start_line_start..start);
        let start_utf8 = self.gather_utf8_len(start_line_start..start);

        let end_line;
        let end_col;
        let end_utf8;
        if end == start {
            end_line = start_line;
            end_col = start_col;
            end_utf8 = start_utf8;
        } else {
            end_line = match self.line_map.binary_search(&end) {
                Ok(i) => i,
                Err(i) => i,
            };
            let end_line_start = if end_line == 0 {
                0
            } else {
                self.line_map[end_line - 1]
            };
            end_col = self.gather_width(end_line_start..end);
            end_utf8 = self.gather_utf8_len(end_line_start..end);
        }

        SourceSpan {
            start_line,
            start_col,
            start_utf8,
            end_line,
            end_col,
            end_utf8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SourceSnippet, SourceSpan};

    #[test]
    fn test_get_line_col() {
        let snippet = SourceSnippet::build_from_utf8(0, b"123\n456", 4);

        assert_eq!(snippet.get_line_col(0), (0, 0));
        assert_eq!(snippet.get_line_col(1), (0, 1));
        assert_eq!(snippet.get_line_col(2), (0, 2));
        assert_eq!(snippet.get_line_col(3), (0, 3));
        assert_eq!(snippet.get_line_col(4), (1, 0));
        assert_eq!(snippet.get_line_col(5), (1, 1));
        assert_eq!(snippet.get_line_col(6), (1, 2));
    }

    #[test]
    fn test_get_line_col_large_meta() {
        let snippet = SourceSnippet::build_from_utf8_ex(
            0,
            b"1\xFF2",
            |_| unreachable!(),
            |_| (true, "\u{A7}".repeat(150)),
            false,
        );

        assert_eq!(snippet.get_line_col(0), (0, 0));
        assert_eq!(snippet.get_line_col(1), (0, 1));
        assert_eq!(snippet.get_line_col(2), (0, 151));
        assert_eq!(snippet.get_line_col(3), (0, 152));
    }

    #[test]
    fn test_convert_span_simple() {
        let snippet = SourceSnippet::build_from_utf8(0, b"123\n456", 4);

        assert_eq!(
            snippet.convert_span(0, 0),
            SourceSpan {
                start_line: 0,
                start_col: 0,
                start_utf8: 0,
                end_line: 0,
                end_col: 0,
                end_utf8: 0,
            },
        );
        assert_eq!(
            snippet.convert_span(0, 1),
            SourceSpan {
                start_line: 0,
                start_col: 0,
                start_utf8: 0,
                end_line: 0,
                end_col: 1,
                end_utf8: 1,
            },
        );
        assert_eq!(
            snippet.convert_span(1, 2),
            SourceSpan {
                start_line: 0,
                start_col: 1,
                start_utf8: 1,
                end_line: 0,
                end_col: 2,
                end_utf8: 2,
            },
        );
        assert_eq!(
            snippet.convert_span(2, 3),
            SourceSpan {
                start_line: 0,
                start_col: 2,
                start_utf8: 2,
                end_line: 0,
                end_col: 3,
                end_utf8: 3,
            },
        );
        assert_eq!(
            snippet.convert_span(3, 4),
            SourceSpan {
                start_line: 0,
                start_col: 3,
                start_utf8: 3,
                end_line: 0,
                end_col: 4,
                end_utf8: 3,
            },
        );
        assert_eq!(
            snippet.convert_span(4, 5),
            SourceSpan {
                start_line: 1,
                start_col: 0,
                start_utf8: 0,
                end_line: 1,
                end_col: 1,
                end_utf8: 1,
            },
        );
        assert_eq!(
            snippet.convert_span(4, 4),
            SourceSpan {
                start_line: 1,
                start_col: 0,
                start_utf8: 0,
                end_line: 1,
                end_col: 0,
                end_utf8: 0,
            },
        );
        assert_eq!(
            snippet.convert_span(5, 6),
            SourceSpan {
                start_line: 1,
                start_col: 1,
                start_utf8: 1,
                end_line: 1,
                end_col: 2,
                end_utf8: 2,
            },
        );
        assert_eq!(
            snippet.convert_span(6, 7),
            SourceSpan {
                start_line: 1,
                start_col: 2,
                start_utf8: 2,
                end_line: 1,
                end_col: 3,
                end_utf8: 3,
            },
        );
        assert_eq!(
            snippet.convert_span(7, 8),
            SourceSpan {
                start_line: 1,
                start_col: 3,
                start_utf8: 3,
                end_line: 1,
                end_col: 3,
                end_utf8: 3,
            },
        );
        assert_eq!(
            snippet.convert_span(8, 9),
            SourceSpan {
                start_line: 1,
                start_col: 3,
                start_utf8: 3,
                end_line: 1,
                end_col: 3,
                end_utf8: 3,
            },
        );
    }

    #[test]
    fn test_convert_span_multi_byte() {
        let snippet = SourceSnippet::build_from_utf8(0, b"1\xEF\xBC\x923\n456", 4);

        assert_eq!(
            snippet.convert_span(0, 1),
            SourceSpan {
                start_line: 0,
                start_col: 0,
                start_utf8: 0,
                end_line: 0,
                end_col: 1,
                end_utf8: 1,
            },
        );
        assert_eq!(
            snippet.convert_span(1, 2),
            SourceSpan {
                start_line: 0,
                start_col: 1,
                start_utf8: 1,
                end_line: 0,
                end_col: 3,
                end_utf8: 4,
            },
        );
        assert_eq!(
            snippet.convert_span(1, 3),
            SourceSpan {
                start_line: 0,
                start_col: 1,
                start_utf8: 1,
                end_line: 0,
                end_col: 3,
                end_utf8: 4,
            },
        );
        assert_eq!(
            snippet.convert_span(1, 4),
            SourceSpan {
                start_line: 0,
                start_col: 1,
                start_utf8: 1,
                end_line: 0,
                end_col: 3,
                end_utf8: 4,
            },
        );
        assert_eq!(
            snippet.convert_span(2, 3),
            SourceSpan {
                start_line: 0,
                start_col: 1,
                start_utf8: 1,
                end_line: 0,
                end_col: 3,
                end_utf8: 4,
            },
        );
        assert_eq!(
            snippet.convert_span(2, 4),
            SourceSpan {
                start_line: 0,
                start_col: 1,
                start_utf8: 1,
                end_line: 0,
                end_col: 3,
                end_utf8: 4,
            },
        );
        assert_eq!(
            snippet.convert_span(3, 4),
            SourceSpan {
                start_line: 0,
                start_col: 1,
                start_utf8: 1,
                end_line: 0,
                end_col: 3,
                end_utf8: 4,
            },
        );
        assert_eq!(
            snippet.convert_span(4, 5),
            SourceSpan {
                start_line: 0,
                start_col: 3,
                start_utf8: 4,
                end_line: 0,
                end_col: 4,
                end_utf8: 5,
            },
        );
        assert_eq!(
            snippet.convert_span(6, 7),
            SourceSpan {
                start_line: 1,
                start_col: 0,
                start_utf8: 0,
                end_line: 1,
                end_col: 1,
                end_utf8: 1,
            },
        );
    }

    #[test]
    fn test_convert_span_invalid_utf8() {
        let snippet = SourceSnippet::build_from_utf8(0, b"1\xFF2\n3", 4);

        assert_eq!(
            snippet.convert_span(0, 1),
            SourceSpan {
                start_line: 0,
                start_col: 0,
                start_utf8: 0,
                end_line: 0,
                end_col: 1,
                end_utf8: 1,
            },
        );
        assert_eq!(
            snippet.convert_span(1, 2),
            SourceSpan {
                start_line: 0,
                start_col: 1,
                start_utf8: 1,
                end_line: 0,
                end_col: 5,
                end_utf8: 5,
            },
        );
        assert_eq!(
            snippet.convert_span(2, 3),
            SourceSpan {
                start_line: 0,
                start_col: 5,
                start_utf8: 5,
                end_line: 0,
                end_col: 6,
                end_utf8: 6,
            },
        );
        assert_eq!(
            snippet.convert_span(4, 5),
            SourceSpan {
                start_line: 1,
                start_col: 0,
                start_utf8: 0,
                end_line: 1,
                end_col: 1,
                end_utf8: 1,
            },
        );
    }

    #[test]
    fn test_convert_span_large_meta() {
        let snippet = SourceSnippet::build_from_utf8_ex(
            0,
            b"1\xFF2",
            |_| unreachable!(),
            |_| (true, "\u{A7}".repeat(150)),
            false,
        );

        assert_eq!(
            snippet.convert_span(0, 1),
            SourceSpan {
                start_line: 0,
                start_col: 0,
                start_utf8: 0,
                end_line: 0,
                end_col: 1,
                end_utf8: 1,
            },
        );
        assert_eq!(
            snippet.convert_span(0, 2),
            SourceSpan {
                start_line: 0,
                start_col: 0,
                start_utf8: 0,
                end_line: 0,
                end_col: 151,
                end_utf8: 301,
            },
        );
        assert_eq!(
            snippet.convert_span(0, 3),
            SourceSpan {
                start_line: 0,
                start_col: 0,
                start_utf8: 0,
                end_line: 0,
                end_col: 152,
                end_utf8: 302,
            },
        );
        assert_eq!(
            snippet.convert_span(1, 2),
            SourceSpan {
                start_line: 0,
                start_col: 1,
                start_utf8: 1,
                end_line: 0,
                end_col: 151,
                end_utf8: 301,
            },
        );
        assert_eq!(
            snippet.convert_span(1, 3),
            SourceSpan {
                start_line: 0,
                start_col: 1,
                start_utf8: 1,
                end_line: 0,
                end_col: 152,
                end_utf8: 302,
            },
        );
        assert_eq!(
            snippet.convert_span(2, 3),
            SourceSpan {
                start_line: 0,
                start_col: 151,
                start_utf8: 301,
                end_line: 0,
                end_col: 152,
                end_utf8: 302,
            },
        );
        assert_eq!(
            snippet.convert_span(2, 2),
            SourceSpan {
                start_line: 0,
                start_col: 151,
                start_utf8: 301,
                end_line: 0,
                end_col: 151,
                end_utf8: 301,
            },
        );
        assert_eq!(
            snippet.convert_span(3, 3),
            SourceSpan {
                start_line: 0,
                start_col: 152,
                start_utf8: 302,
                end_line: 0,
                end_col: 152,
                end_utf8: 302,
            },
        );
    }
}
