use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

mod chars;
mod latin1;
mod utf8;

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
    src_line_map: Vec<usize>,
    utf8_line_map: Vec<usize>,
    metas: Vec<UnitMeta>,
    large_widths: Vec<(usize, usize)>,
    large_utf8_lens: Vec<(usize, usize)>,
}

#[derive(Clone, PartialEq, Eq)]
struct UnitMeta {
    inner: u16,
}

impl core::fmt::Debug for UnitMeta {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.is_extra() {
            f.write_str("UnitMeta::extra()")
        } else {
            f.debug_struct("UnitMeta")
                .field("width", &self.width())
                .field("utf8_len", &self.utf8_len())
                .field("alt", &self.is_alt())
                .finish()
        }
    }
}

impl UnitMeta {
    const EXTRA_MASK: u16 = 0x8000;
    const MAX_WIDTH: u8 = 0x7F;
    const MAX_UTF8_LEN: u8 = 0x7F;
    const ALT_MASK: u16 = 0x0080;

    #[inline]
    fn extra() -> Self {
        Self {
            inner: Self::EXTRA_MASK,
        }
    }

    #[inline]
    fn new(width: u8, utf8_len: u8, alt: bool) -> Self {
        assert!(width <= Self::MAX_WIDTH);
        assert!(utf8_len <= Self::MAX_UTF8_LEN);
        let alt = if alt { Self::ALT_MASK } else { 0 };
        Self {
            inner: u16::from(width) | (u16::from(utf8_len) << 8) | alt,
        }
    }

    #[inline]
    fn is_extra(&self) -> bool {
        self.inner & Self::EXTRA_MASK != 0
    }

    #[inline]
    fn width(&self) -> u8 {
        (self.inner as u8) & Self::MAX_WIDTH
    }

    #[inline]
    fn utf8_len(&self) -> u8 {
        ((self.inner >> 8) as u8) & Self::MAX_UTF8_LEN
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
    pub(crate) start_utf8: usize,
    pub(crate) end_line: usize,
    pub(crate) end_col: usize,
    pub(crate) end_utf8: usize,
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
        match self.src_line_map.binary_search(&pos) {
            Ok(i) => i + 1,
            Err(i) => i,
        }
    }

    pub(crate) fn src_line_range(&self, line_i: usize) -> core::ops::Range<usize> {
        let start = if line_i == 0 {
            0
        } else {
            self.src_line_map[line_i - 1]
        };
        let end = if line_i == self.src_line_map.len() {
            self.metas.len()
        } else {
            self.src_line_map[line_i]
        };
        start..end
    }

    pub(crate) fn utf8_line_range(&self, line_i: usize) -> core::ops::Range<usize> {
        let start = if line_i == 0 {
            0
        } else {
            self.utf8_line_map[line_i - 1]
        };
        let end = if line_i == self.utf8_line_map.len() {
            self.utf8_text.len()
        } else {
            self.utf8_line_map[line_i]
        };
        start..end
    }

    fn utf8_lens(&self, src_range: core::ops::Range<usize>) -> impl Iterator<Item = usize> + '_ {
        let range_start = src_range.start;
        self.metas[src_range]
            .iter()
            .enumerate()
            .map(move |(i, meta)| {
                let len_i = meta.utf8_len();
                if len_i == UnitMeta::MAX_UTF8_LEN {
                    let large_i = self
                        .large_utf8_lens
                        .binary_search_by_key(&(i + range_start), |&(j, _)| j)
                        .unwrap();
                    self.large_utf8_lens[large_i].1
                } else {
                    usize::from(len_i)
                }
            })
    }

    fn gather_utf8_len(&self, src_range: core::ops::Range<usize>) -> usize {
        self.utf8_lens(src_range).sum()
    }

    fn widths(&self, src_range: core::ops::Range<usize>) -> impl Iterator<Item = usize> + '_ {
        let range_start = src_range.start;
        self.metas[src_range]
            .iter()
            .enumerate()
            .map(move |(i, meta)| {
                let width_i = meta.width();
                if width_i == UnitMeta::MAX_WIDTH {
                    let large_i = self
                        .large_widths
                        .binary_search_by_key(&(i + range_start), |&(j, _)| j)
                        .unwrap();
                    self.large_widths[large_i].1
                } else {
                    usize::from(width_i)
                }
            })
    }

    fn gather_width(&self, src_range: core::ops::Range<usize>) -> usize {
        self.widths(src_range).sum()
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

        let start_line = match self.src_line_map.binary_search(&start) {
            Ok(i) => i + 1,
            Err(i) => i,
        };
        let start_line_start = if start_line == 0 {
            0
        } else {
            self.src_line_map[start_line - 1]
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
            end_line = match self.src_line_map.binary_search(&end) {
                Ok(i) => i,
                Err(i) => i,
            };
            let end_line_start = if end_line == 0 {
                0
            } else {
                self.src_line_map[end_line - 1]
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

/// Style for how control characters should be represented in a snippet.
///
/// This style is applied by [`SnippetBuilder::maybe_push_control_char()`],
/// which renders certain control and "invisible" characters in a safe, explicit
/// way.
///
/// # Rendering rules
///
/// - Tab (U+0009): pushes `tab_width` spaces. `alt` is ignored (treated as
///   `false`).
/// - C0 controls (U+0000 to U+001F, excluding tab) and DEL (U+007F):
///   - [`ControlCharStyle::Replacement`]: Unicode Control Pictures (␀, ␁, ...).
///   - [`ControlCharStyle::Hexadecimal`]: `<XX>` (two hex digits).
/// - C1 controls (U+0080 to U+009F): always `<XX>`.
/// - ZERO WIDTH JOINER (U+200D): pushes nothing (but still accounts for
///   `orig_len`).
/// - Bidirectional text control characters (U+202A to U+202E, U+2066 to U+2069):
///   `<XXXX>` (four hex digits).
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
    src_line_map: Vec<usize>,
    utf8_line_map: Vec<usize>,
    metas: Vec<UnitMeta>,
    large_widths: Vec<(usize, usize)>,
    large_utf8_lens: Vec<(usize, usize)>,
}

impl SnippetBuilder {
    fn new(start_line: usize) -> Self {
        Self {
            start_line,
            utf8_text: String::new(),
            src_line_map: Vec::new(),
            utf8_line_map: Vec::new(),
            metas: Vec::new(),
            large_widths: Vec::new(),
            large_utf8_lens: Vec::new(),
        }
    }

    /// Finalizes the builder and returns the constructed [`Snippet`].
    pub fn finish(self) -> Snippet {
        Snippet {
            start_line: self.start_line,
            utf8_text: self.utf8_text.into_boxed_str(),
            src_line_map: self.src_line_map,
            utf8_line_map: self.utf8_line_map,
            metas: self.metas,
            large_widths: self.large_widths,
            large_utf8_lens: self.large_utf8_lens,
        }
    }

    /// Ends the current line and starts a new one.
    ///
    /// `orig_len` is the number of *source units* consumed by the line break.
    /// For example, it can be `1` for `"\n"` or `2` for `"\r\n"`.
    pub fn next_line(&mut self, orig_len: usize) {
        self.push_meta(orig_len, 1, 0, false);
        self.src_line_map.push(self.metas.len());
        self.utf8_line_map.push(self.utf8_text.len());
    }

    /// Consumes `orig_len` source units without producing any rendered text.
    ///
    /// This is useful when you need to "eat" units that should not be visible
    /// in the output but still need to be span-addressable.
    pub fn push_empty(&mut self, orig_len: usize) {
        self.push_meta(orig_len, 0, 0, false);
    }

    /// Appends `text` to the current line.
    pub fn push_str(&mut self, text: &str, orig_len: usize, alt: bool) {
        self.utf8_text.push_str(text);
        self.push_meta(orig_len, string_width(text), text.len(), alt);
    }

    /// Appends a single character to the current line.
    pub fn push_char(&mut self, chr: char, orig_len: usize, alt: bool) {
        self.utf8_text.push(chr);
        self.push_meta(orig_len, char_width(chr), chr.len_utf8(), alt);
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
        self.push_meta(orig_len, width, width, alt);
    }

    /// Writes formatted text to the current line.
    pub fn push_fmt(&mut self, args: core::fmt::Arguments<'_>, orig_len: usize, alt: bool) {
        let old_text_len = self.utf8_text.len();
        core::fmt::write(&mut self.utf8_text, args)
            .expect("a format implementation returned an error unexpectedly");
        let new_text_len = self.utf8_text.len();
        let new_text = &self.utf8_text[old_text_len..new_text_len];
        self.push_meta(orig_len, string_width(new_text), new_text.len(), alt);
    }

    /// Pushes a visible representation of certain control/invisible characters.
    ///
    /// This ensures that characters that are typically invisible (or can affect
    /// layout) are rendered in a safe, explicit way.
    ///
    /// If `chr` matches one of the handled cases, this method pushes a replacement
    /// representation (which may be empty) and returns `true`. Otherwise it leaves
    /// the builder unchanged and returns `false`. The exact replacement rules are
    /// documented on [`ControlCharStyle`].
    ///
    /// # Example
    ///
    /// ```
    /// # let mut builder = sourceannot::Snippet::builder(0);
    /// # let chr = '\t';
    /// // Assume `chr` is `char` from a UTF-8 source
    /// let is_control = builder.maybe_push_control_char(
    ///     chr,
    ///     chr.len_utf8(),
    ///     4,
    ///     sourceannot::ControlCharStyle::Hexadecimal,
    ///     true,
    /// );
    /// if !is_control {
    ///     // If it is not a control character, push it as-is
    ///     builder.push_char(chr, chr.len_utf8(), false);
    /// }
    /// ```
    pub fn maybe_push_control_char(
        &mut self,
        chr: char,
        orig_len: usize,
        tab_width: usize,
        style: ControlCharStyle,
        alt: bool,
    ) -> bool {
        if chr == '\t' {
            self.push_spaces(tab_width, orig_len, false);
            return true;
        }

        if style == ControlCharStyle::Replacement {
            if matches!(chr, '\u{00}'..='\u{1F}') {
                let replacement = char::try_from(u32::from(chr) + 0x2400).unwrap();
                self.push_char(replacement, orig_len, alt);
                return true;
            } else if chr == '\u{7F}' {
                let replacement = '␡';
                self.push_char(replacement, orig_len, alt);
                return true;
            }
        }

        if matches!(chr, '\u{00}'..='\u{1F}' | '\u{7F}'..='\u{9F}') {
            self.push_fmt(format_args!("<{:02X}>", u32::from(chr)), orig_len, alt);
            true
        } else if chr == '\u{200D}' {
            // Replace ZERO WIDTH JOINER with nothing
            self.push_empty(orig_len);
            true
        } else if matches!(chr, '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}') {
            // Replace bidirectional text control characters
            self.push_fmt(format_args!("<{:04X}>", u32::from(chr)), orig_len, alt);
            true
        } else {
            false
        }
    }

    fn push_meta(&mut self, orig_len: usize, width: usize, utf8_len: usize, alt: bool) {
        if orig_len == 0 {
            return;
        }

        let meta_width = if width >= usize::from(UnitMeta::MAX_WIDTH) {
            self.large_widths.push((self.metas.len(), width));
            UnitMeta::MAX_WIDTH
        } else {
            width as u8
        };
        let meta_utf8_len = if utf8_len >= usize::from(UnitMeta::MAX_UTF8_LEN) {
            self.large_utf8_lens.push((self.metas.len(), utf8_len));
            UnitMeta::MAX_UTF8_LEN
        } else {
            utf8_len as u8
        };
        self.metas
            .push(UnitMeta::new(meta_width, meta_utf8_len, alt));
        for _ in 1..orig_len {
            // Each element of `self.metas` corresponds to a unit in the original
            // source, so fill with "extras" for multi-unit chunks (for example, a
            // multi-byte UTF-8 character, a multi-byte invalid UTF-8 sequence or
            // a CRLF line break).
            self.metas.push(UnitMeta::extra());
        }
    }
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
