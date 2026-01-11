use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

mod chars;
mod latin1;
mod utf8;
mod utils;

pub use utf8::InvalidUtf8SeqStyle;

/// A snippet of source code prepared for annotated rendering.
///
/// # Source units and spans
///
/// Annotation spans are `Range<usize>` indices into the snippet's *source unit*
/// sequence. The exact meaning of a "unit" depends on how you create the
/// snippet:
///
/// - [`Snippet::with_utf8()`], [`Snippet::with_utf8_bytes()`] and
///   [`Snippet::with_latin1()`] treat a unit as a **byte** in the original byte
///   sequence. In the UTF-8 case, a valid printable character may correspond to
///   1 to 4 source units.
/// - [`Snippet::with_chars()`] treats a unit as a **[`char`]** in the original
///   character sequence.
/// - [`Snippet::builder()`] allows units to be defined by the caller.
///
/// Because the snippet may render replacements (expanded tabs, control-picture
/// glyphs, `<XX>` escapes, etc.), source-unit indices are *not* indices into the
/// final rendered UTF-8 text. The snippet keeps the necessary mapping so spans
/// still line up with what is shown.
///
/// # Alternate text
///
/// Some rendered fragments can be marked as "alternate" (for example, a
/// control-character replacement). Renderers can use this to present those
/// fragments differently (e.g., highlight them).
#[derive(Clone, Debug)]
pub struct Snippet {
    start_line: usize,
    utf8_text: Box<str>,
    line_map: Vec<(usize, usize)>, // (src, utf8)
    metas: Vec<UnitMeta>,
    large_utf8_lens: Vec<(usize, usize)>,
}

#[derive(Clone, PartialEq, Eq)]
struct UnitMeta {
    inner: u8,
}

impl core::fmt::Debug for UnitMeta {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.is_extra() {
            f.write_str("UnitMeta::extra()")
        } else {
            f.debug_struct("UnitMeta")
                .field("utf8_len", &self.utf8_len())
                .field("alt", &self.is_alt())
                .finish()
        }
    }
}

impl UnitMeta {
    const EXTRA_MASK: u8 = 0x80;
    const ALT_MASK: u8 = 0x40;
    const MAX_UTF8_LEN: u8 = 0x3F;

    #[inline]
    fn extra() -> Self {
        Self {
            inner: Self::EXTRA_MASK,
        }
    }

    #[inline]
    fn new(utf8_len: u8, alt: bool) -> Self {
        assert!(utf8_len <= Self::MAX_UTF8_LEN);
        let alt = if alt { Self::ALT_MASK } else { 0 };
        Self {
            inner: utf8_len | alt,
        }
    }

    #[inline]
    fn is_extra(&self) -> bool {
        self.inner & Self::EXTRA_MASK != 0
    }

    #[inline]
    fn utf8_len(&self) -> u8 {
        self.inner & Self::MAX_UTF8_LEN
    }

    #[inline]
    fn is_alt(&self) -> bool {
        self.inner & Self::ALT_MASK != 0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SourceSpan {
    pub(crate) start_line: usize,
    pub(crate) start_col: usize,
    pub(crate) start_col_utf8: usize,
    pub(crate) end_line: usize,
    pub(crate) end_col: usize,
    pub(crate) end_col_utf8: usize,
}

impl Snippet {
    /// Creates a builder for manually constructing a snippet.
    ///
    /// `start_line` is the line number to associate with the first rendered
    /// line of the snippet. This is typically 1 when the snippet corresponds
    /// to a whole source file.
    ///
    /// See [`SnippetBuilder`] for details on how to use the builder.
    pub fn builder(start_line: usize) -> SnippetBuilder {
        SnippetBuilder::new(start_line)
    }

    pub(crate) fn src_pos_to_line(&self, pos: usize) -> usize {
        match self.line_map.binary_search_by_key(&pos, |&(i, _)| i) {
            Ok(i) => i + 1,
            Err(i) => i,
        }
    }

    pub(crate) fn src_line_range(&self, line_i: usize) -> core::ops::Range<usize> {
        let start = if line_i == 0 {
            0
        } else {
            self.line_map[line_i - 1].0
        };
        let end = if line_i == self.line_map.len() {
            self.metas.len()
        } else {
            self.line_map[line_i].0
        };
        start..end
    }

    pub(crate) fn utf8_line_range(&self, line_i: usize) -> core::ops::Range<usize> {
        let start = if line_i == 0 {
            0
        } else {
            self.line_map[line_i - 1].1
        };
        let end = if line_i == self.line_map.len() {
            self.utf8_text.len()
        } else {
            self.line_map[line_i].1
        };
        start..end
    }

    fn gather_utf8_len(&self, src_range: core::ops::Range<usize>) -> (usize, bool) {
        let range_start = src_range.start;
        let mut len_sum = 0;
        let mut last_is_zero = false;
        for (i, meta) in self.metas[src_range].iter().enumerate() {
            let meta_len = meta.utf8_len();
            let len = if meta_len == UnitMeta::MAX_UTF8_LEN {
                let large_i = self
                    .large_utf8_lens
                    .binary_search_by_key(&(i + range_start), |&(j, _)| j)
                    .unwrap();
                self.large_utf8_lens[large_i].1
            } else {
                usize::from(meta_len)
            };
            len_sum += len;
            if !meta.is_extra() {
                last_is_zero = len == 0;
            }
        }
        (len_sum, last_is_zero)
    }

    pub(crate) fn utf8_lens_and_alts(
        &self,
        src_range: core::ops::Range<usize>,
    ) -> impl Iterator<Item = (usize, bool)> + '_ {
        let range_start = src_range.start;
        self.metas[src_range]
            .iter()
            .enumerate()
            .map(move |(i, meta)| {
                let len_i = meta.utf8_len();
                let alt = meta.is_alt();
                let utf8_len = if len_i == UnitMeta::MAX_UTF8_LEN {
                    let large_i = self
                        .large_utf8_lens
                        .binary_search_by_key(&(i + range_start), |&(j, _)| j)
                        .unwrap();
                    self.large_utf8_lens[large_i].1
                } else {
                    usize::from(len_i)
                };
                (utf8_len, alt)
            })
    }

    #[inline]
    pub(crate) fn start_line(&self) -> usize {
        self.start_line
    }

    #[inline]
    pub(crate) fn utf8_line_text(&self, i: usize) -> &str {
        &self.utf8_text[self.utf8_line_range(i)]
    }

    pub(crate) fn convert_span(&self, mut start: usize, mut end: usize) -> SourceSpan {
        end = end.max(start);

        while self.metas.get(start).is_some_and(UnitMeta::is_extra) {
            start -= 1;
        }
        while self.metas.get(end).is_some_and(UnitMeta::is_extra) {
            end += 1;
        }
        start = start.min(self.metas.len());
        end = end.min(self.metas.len());

        let start_line = match self.line_map.binary_search_by_key(&start, |&(i, _)| i) {
            Ok(i) => i + 1,
            Err(i) => i,
        };
        let (start_line_src_start, start_line_utf8_start) = if start_line == 0 {
            (0, 0)
        } else {
            self.line_map[start_line - 1]
        };
        let (start_col_utf8, _) = self.gather_utf8_len(start_line_src_start..start);
        let start_col = string_width(&self.utf8_text[start_line_utf8_start..][..start_col_utf8]);

        let end_line;
        let mut end_col;
        let mut end_col_utf8;
        if end == start {
            end_line = start_line;
            end_col = start_col + 1; // Draw at least one caret for zero-width spans
            end_col_utf8 = start_col_utf8;
        } else {
            end_line = match self.line_map.binary_search_by_key(&end, |&(i, _)| i) {
                Ok(i) => i,
                Err(i) => i,
            };

            let last_width_is_zero;
            if end_line == start_line {
                (end_col_utf8, last_width_is_zero) = self.gather_utf8_len(start..end);
                end_col_utf8 += start_col_utf8;
                end_col = start_col
                    + string_width(
                        &self.utf8_text[start_line_utf8_start..][start_col_utf8..end_col_utf8],
                    );
            } else {
                let (end_line_src_start, end_line_utf8_start) = if end_line == 0 {
                    (0, 0)
                } else {
                    self.line_map[end_line - 1]
                };
                (end_col_utf8, last_width_is_zero) = self.gather_utf8_len(end_line_src_start..end);
                end_col = string_width(&self.utf8_text[end_line_utf8_start..][..end_col_utf8]);
            }

            if last_width_is_zero {
                // If the last element pointed to by the span has zero width,
                // add one caret to account for it.
                end_col += 1;
            }
        }

        SourceSpan {
            start_line,
            start_col,
            start_col_utf8,
            end_line,
            end_col,
            end_col_utf8,
        }
    }
}

/// Style for how control characters should be represented in a snippet.
///
/// This determines how functions like [`Snippet::with_utf8()`],
/// [`Snippet::with_latin1()`], etc. handle control characters.
///
/// The documentation of functions that takes a [`ControlCharStyle`]
/// describes in detail how control characters are represented.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ControlCharStyle {
    Replacement,
    Hexadecimal,
}

/// Incrementally constructs a [`Snippet`].
///
/// This type is the low-level building block that allows to create snippets in
/// a custom way, with more flexibility than the convenience methods of [`Snippet`].
///
/// Every time you push something into the builder you must tell it how many
/// *source units* that rendered fragment corresponds to. That is the `orig_len`
/// parameter accepted by [`next_line()`](SnippetBuilder::next_line) and the
/// `push_*` methods.
///
/// If you want a fragment to be addressable individually by a span, push it
/// with its own `push_*` call. For example, if you call
/// `push_str("abc", 3, false)`, any span that covers any of those three units
/// will cover the entire rendered `"abc"` fragment.
///
/// Most `push_*` methods also accept an `alt` flag. When `alt` is `true`, the
/// UTF-8 byte range appended to the current line is recorded as "alternate"
/// text. This is intended for replacement/escaped representations (for example,
/// `<XX>` escapes for control characters or invalid bytes) so it can be
/// rendered differently (e.g., highlighted).
///
/// # Example
///
/// ```
/// /// Builds a snippet from ASCII source bytes.
/// ///
/// /// `"\n"` and `"\r\n"` are treated as line breaks and tabs are expanded to
/// /// 4 spaces. Control and non-ASCII characters are represented as `<XX>`.
/// fn build_ascii_snippet(source: &[u8]) -> sourceannot::Snippet {
///     let mut builder = sourceannot::Snippet::builder(1);
///     let mut rest = source;
///     while let Some((&byte, new_rest)) = rest.split_first() {
///         rest = new_rest;
///         if byte == b'\n' {
///             // `"\n"` line break
///             builder.next_line(1);
///         } else if byte == b'\r' {
///             if let Some(new_rest) = rest.strip_prefix(b"\n") {
///                 // `"\r\n"` line break
///                 rest = new_rest;
///                 builder.next_line(2);
///             } else {
///                 // Lone `"\r"`, treat as a control character
///                 builder.push_str("<0D>", 1, true);
///             }
///         } else if byte == b'\t' {
///             // Tab as 4 spaces
///             builder.push_spaces(4, 1, false);
///         } else if matches!(byte, b' '..=b'~') {
///             // Printable ASCII
///             builder.push_char(byte.into(), 1, false);
///         } else {
///             // Control or non-ASCII
///             builder.push_fmt(format_args!("<{byte:02X}>"), 1, true);
///         }
///     }
///     builder.finish()
/// }
/// ```
pub struct SnippetBuilder {
    start_line: usize,
    utf8_text: String,
    line_map: Vec<(usize, usize)>,
    metas: Vec<UnitMeta>,
    large_utf8_lens: Vec<(usize, usize)>,
}

impl SnippetBuilder {
    fn new(start_line: usize) -> Self {
        Self {
            start_line,
            utf8_text: String::new(),
            line_map: Vec::new(),
            metas: Vec::new(),
            large_utf8_lens: Vec::new(),
        }
    }

    /// Finalizes the builder and returns the constructed [`Snippet`].
    pub fn finish(self) -> Snippet {
        Snippet {
            start_line: self.start_line,
            utf8_text: self.utf8_text.into_boxed_str(),
            line_map: self.line_map,
            metas: self.metas,
            large_utf8_lens: self.large_utf8_lens,
        }
    }

    /// Ends the current line and starts a new one.
    ///
    /// `orig_len` is the number of *source units* consumed by the line break.
    /// For example, it can be `1` for `"\n"` or `2` for `"\r\n"`.
    pub fn next_line(&mut self, orig_len: usize) {
        self.push_meta(orig_len, 0, false);
        self.line_map.push((self.metas.len(), self.utf8_text.len()));
    }

    /// Consumes `orig_len` source units without producing any rendered text.
    ///
    /// This is useful when you need to "eat" units that should not be visible
    /// in the output but still need to be span-addressable.
    pub fn push_empty(&mut self, orig_len: usize) {
        self.push_meta(orig_len, 0, false);
    }

    /// Appends `text` to the current line.
    pub fn push_str(&mut self, text: &str, orig_len: usize, alt: bool) {
        self.utf8_text.push_str(text);
        self.push_meta(orig_len, text.len(), alt);
    }

    /// Appends a single character to the current line.
    pub fn push_char(&mut self, chr: char, orig_len: usize, alt: bool) {
        self.utf8_text.push(chr);
        self.push_meta(orig_len, chr.len_utf8(), alt);
    }

    /// Appends `width` ASCII spaces to the current line.
    pub fn push_spaces(&mut self, width: usize, orig_len: usize, alt: bool) {
        let spaces = "                ";
        let mut rem = width;
        while rem != 0 {
            let n = rem.min(spaces.len());
            self.utf8_text.push_str(&spaces[..n]);
            rem -= n;
        }
        self.push_meta(orig_len, width, alt);
    }

    /// Writes formatted text to the current line.
    pub fn push_fmt(&mut self, args: core::fmt::Arguments<'_>, orig_len: usize, alt: bool) {
        let old_text_len = self.utf8_text.len();
        core::fmt::write(&mut self.utf8_text, args)
            .expect("a format implementation returned an error unexpectedly");
        let new_text_len = self.utf8_text.len();
        let new_text = &self.utf8_text[old_text_len..new_text_len];
        self.push_meta(orig_len, new_text.len(), alt);
    }

    fn push_meta(&mut self, orig_len: usize, utf8_len: usize, alt: bool) {
        if orig_len == 0 {
            return;
        }

        let meta_utf8_len = if utf8_len >= usize::from(UnitMeta::MAX_UTF8_LEN) {
            self.large_utf8_lens.push((self.metas.len(), utf8_len));
            UnitMeta::MAX_UTF8_LEN
        } else {
            utf8_len as u8
        };
        self.metas.push(UnitMeta::new(meta_utf8_len, alt));
        for _ in 1..orig_len {
            // Each element of `self.metas` corresponds to a unit in the original
            // source, so fill with "extras" for multi-unit chunks (for example, a
            // multi-byte UTF-8 character, a multi-byte invalid UTF-8 sequence or
            // a CRLF line break).
            self.metas.push(UnitMeta::extra());
        }
    }
}

#[inline]
pub fn char_should_be_replaced(chr: char) -> bool {
    matches!(
        chr,
        '\u{00}'..='\u{1F}' // C0 controls
        | '\u{7F}'..='\u{9F}' // DEL and C1 controls
        | '\u{200D}' // ZERO WIDTH JOINER
        | '\u{202A}'..='\u{202E}' // Bidirectional text controls
        | '\u{2066}'..='\u{2069}' // More bidirectional text controls
    )
}

fn string_width(s: &str) -> usize {
    s.chars().map(char_width).sum()
}

fn char_width(ch: char) -> usize {
    unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::{Snippet, SourceSpan};

    #[test]
    fn test_src_pos_to_line() {
        let mut builder = Snippet::builder(0);
        builder.push_char('1', 1, false);
        builder.push_char('2', 1, false);
        builder.push_char('3', 1, false);
        builder.next_line(1);
        builder.push_char('4', 1, false);
        builder.push_char('5', 1, false);
        builder.push_char('6', 1, false);
        let snippet = builder.finish();

        assert_eq!(snippet.src_pos_to_line(0), (0));
        assert_eq!(snippet.src_pos_to_line(1), (0));
        assert_eq!(snippet.src_pos_to_line(2), (0));
        assert_eq!(snippet.src_pos_to_line(3), (0));
        assert_eq!(snippet.src_pos_to_line(4), (1));
        assert_eq!(snippet.src_pos_to_line(5), (1));
        assert_eq!(snippet.src_pos_to_line(6), (1));
    }

    #[test]
    fn test_src_pos_to_line_large_meta() {
        let mut builder = Snippet::builder(0);
        builder.push_char('1', 1, false);
        builder.push_str(&"\u{A7}".repeat(150), 150, false);
        let snippet = builder.finish();

        assert_eq!(snippet.src_pos_to_line(0), (0));
        assert_eq!(snippet.src_pos_to_line(1), (0));
        assert_eq!(snippet.src_pos_to_line(2), (0));
        assert_eq!(snippet.src_pos_to_line(3), (0));
    }

    #[test]
    fn test_convert_span() {
        let mut builder = Snippet::builder(0);
        builder.push_char('1', 1, false);
        builder.push_char('\u{FF12}', 3, false);
        builder.push_char('3', 1, false);
        builder.next_line(1);
        builder.push_char('4', 1, false);
        builder.push_char('5', 1, false);
        builder.push_char('6', 1, false);
        let snippet = builder.finish();

        assert_eq!(
            snippet.convert_span(0, 1),
            SourceSpan {
                start_line: 0,
                start_col: 0,
                start_col_utf8: 0,
                end_line: 0,
                end_col: 1,
                end_col_utf8: 1,
            },
        );
        assert_eq!(
            snippet.convert_span(1, 2),
            SourceSpan {
                start_line: 0,
                start_col: 1,
                start_col_utf8: 1,
                end_line: 0,
                end_col: 3,
                end_col_utf8: 4,
            },
        );
        assert_eq!(
            snippet.convert_span(1, 3),
            SourceSpan {
                start_line: 0,
                start_col: 1,
                start_col_utf8: 1,
                end_line: 0,
                end_col: 3,
                end_col_utf8: 4,
            },
        );
        assert_eq!(
            snippet.convert_span(1, 4),
            SourceSpan {
                start_line: 0,
                start_col: 1,
                start_col_utf8: 1,
                end_line: 0,
                end_col: 3,
                end_col_utf8: 4,
            },
        );
        assert_eq!(
            snippet.convert_span(2, 3),
            SourceSpan {
                start_line: 0,
                start_col: 1,
                start_col_utf8: 1,
                end_line: 0,
                end_col: 3,
                end_col_utf8: 4,
            },
        );
        assert_eq!(
            snippet.convert_span(2, 4),
            SourceSpan {
                start_line: 0,
                start_col: 1,
                start_col_utf8: 1,
                end_line: 0,
                end_col: 3,
                end_col_utf8: 4,
            },
        );
        assert_eq!(
            snippet.convert_span(3, 4),
            SourceSpan {
                start_line: 0,
                start_col: 1,
                start_col_utf8: 1,
                end_line: 0,
                end_col: 3,
                end_col_utf8: 4,
            },
        );
        assert_eq!(
            snippet.convert_span(4, 5),
            SourceSpan {
                start_line: 0,
                start_col: 3,
                start_col_utf8: 4,
                end_line: 0,
                end_col: 4,
                end_col_utf8: 5,
            },
        );
        assert_eq!(
            snippet.convert_span(6, 7),
            SourceSpan {
                start_line: 1,
                start_col: 0,
                start_col_utf8: 0,
                end_line: 1,
                end_col: 1,
                end_col_utf8: 1,
            },
        );
    }

    #[test]
    fn test_convert_span_large_meta() {
        let mut builder = Snippet::builder(0);
        builder.push_char('1', 1, false);
        builder.push_str(&"\u{A7}".repeat(150), 1, false);
        builder.push_char('2', 1, false);
        let snippet = builder.finish();

        assert_eq!(
            snippet.convert_span(0, 1),
            SourceSpan {
                start_line: 0,
                start_col: 0,
                start_col_utf8: 0,
                end_line: 0,
                end_col: 1,
                end_col_utf8: 1,
            },
        );
        assert_eq!(
            snippet.convert_span(0, 2),
            SourceSpan {
                start_line: 0,
                start_col: 0,
                start_col_utf8: 0,
                end_line: 0,
                end_col: 151,
                end_col_utf8: 301,
            },
        );
        assert_eq!(
            snippet.convert_span(0, 3),
            SourceSpan {
                start_line: 0,
                start_col: 0,
                start_col_utf8: 0,
                end_line: 0,
                end_col: 152,
                end_col_utf8: 302,
            },
        );
        assert_eq!(
            snippet.convert_span(1, 2),
            SourceSpan {
                start_line: 0,
                start_col: 1,
                start_col_utf8: 1,
                end_line: 0,
                end_col: 151,
                end_col_utf8: 301,
            },
        );
        assert_eq!(
            snippet.convert_span(1, 3),
            SourceSpan {
                start_line: 0,
                start_col: 1,
                start_col_utf8: 1,
                end_line: 0,
                end_col: 152,
                end_col_utf8: 302,
            },
        );
        assert_eq!(
            snippet.convert_span(2, 3),
            SourceSpan {
                start_line: 0,
                start_col: 151,
                start_col_utf8: 301,
                end_line: 0,
                end_col: 152,
                end_col_utf8: 302,
            },
        );
        assert_eq!(
            snippet.convert_span(2, 2),
            SourceSpan {
                start_line: 0,
                start_col: 151,
                start_col_utf8: 301,
                end_line: 0,
                end_col: 152,
                end_col_utf8: 301,
            },
        );
        assert_eq!(
            snippet.convert_span(3, 3),
            SourceSpan {
                start_line: 0,
                start_col: 152,
                start_col_utf8: 302,
                end_line: 0,
                end_col: 153,
                end_col_utf8: 302,
            },
        );
    }
}
