use alloc::string::String;
use alloc::vec::Vec;

mod latin1;
mod utf8;

pub use latin1::Latin1Snippet;
pub use utf8::{InvalidUtf8SeqStyle, Utf8Snippet};

/// Style for rendering control characters (except tabs and line breaks).
///
/// This style is applied by [`SnippetLineBuilder::maybe_push_control_char()`],
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

/// A mapping from source positions to line/column numbers.
///
/// The mapping is defined in terms of *units*:
/// - For byte-based snippets, a unit is a byte index.
/// - For char-based snippets, a unit is an index into the `char` slice.
/// - For custom snippets, units are whatever your [`Snippet`] uses.
///
/// Line indices and column offsets returned by this type are **0-based**.
///
/// A source always has at least one line, even if it is empty. When it ends
/// with a line terminator, there is an empty last line.
pub struct LineMap {
    total_len: usize,
    line_map: Vec<usize>,
}

impl LineMap {
    /// Creates a [`LineMapBuilder`] for a source of length `total_len`.
    ///
    /// Use this if you need to build a [`LineMap`] while scanning input
    /// incrementally or with custom line-break rules.
    #[inline]
    pub fn builder(total_len: usize) -> LineMapBuilder {
        LineMapBuilder::new(total_len)
    }

    /// Builds a [`LineMap`] from a slice of units.
    ///
    /// Line breaks are detected as:
    /// - `lf` (`"\n"` in typical text),
    /// - `cr` followed by `lf` (`"\r\n"`),
    /// - and bare `cr` (`"\r"`) when `cr_is_eol` is `true`.
    pub fn with_slice<U: PartialEq>(src: &[U], lf: U, cr: U, cr_is_eol: bool) -> Self {
        let mut builder = Self::builder(src.len());
        let mut rem = src;
        while let Some((unit, new_rem)) = rem.split_first() {
            rem = new_rem;
            let mut new_line = false;
            if *unit == lf {
                new_line = true;
            } else if *unit == cr {
                if let Some((_, new_rem2)) = rem.split_first().filter(|&(c, _)| *c == lf) {
                    rem = new_rem2;
                    new_line = true;
                } else if cr_is_eol {
                    new_line = true;
                }
            }
            if new_line {
                builder.add_line(src.len() - rem.len());
            }
        }
        builder.finish()
    }

    /// Convenience wrapper around [`LineMap::with_slice`] for byte sources.
    pub fn with_bytes(src: &[u8], cr_is_eol: bool) -> Self {
        Self::with_slice(src, b'\n', b'\r', cr_is_eol)
    }

    /// Convenience wrapper around [`LineMap::with_slice`] for `char` sources.
    pub fn with_chars(src: &[char], cr_is_eol: bool) -> Self {
        Self::with_slice(src, '\n', '\r', cr_is_eol)
    }

    /// Returns the unit range for the given (0-based) line index.
    ///
    /// [`LineMap::line_range`] returns a range that includes the line terminator
    /// (`"\n"`, `"\r"`, or `"\r\n"`) for all lines except the last one.
    pub fn line_range(&self, line_i: usize) -> core::ops::Range<usize> {
        assert!(line_i < self.num_lines());
        let start = if line_i == 0 {
            0
        } else {
            self.line_map[line_i - 1]
        };
        let end = self.line_map.get(line_i).copied().unwrap_or(self.total_len);
        start..end
    }

    #[inline]
    /// Returns the number of lines in the source.
    ///
    /// The value is always at least 1, even for an empty source.
    pub fn num_lines(&self) -> usize {
        self.line_map.len() + 1
    }

    /// Converts a unit position into a (0-based) `(line_index, column_offset)`.
    pub fn pos_to_line_col(&self, pos: usize) -> (usize, usize) {
        let line_i = match self.line_map.binary_search(&pos) {
            Ok(i) => i + 1,
            Err(i) => i,
        };
        let line_start = if line_i == 0 {
            0
        } else {
            self.line_map[line_i - 1]
        };
        (line_i, pos - line_start)
    }
}

/// Incremental builder for a [`LineMap`].
///
/// The builder stores the start position of each line after the first.
/// Positions must be strictly increasing and within `0..=total_len`.
pub struct LineMapBuilder {
    total_len: usize,
    line_map: Vec<usize>,
}

impl LineMapBuilder {
    #[inline]
    fn new(total_len: usize) -> Self {
        Self {
            total_len,
            line_map: Vec::new(),
        }
    }

    /// Adds a new line start at unit position `pos`.
    ///
    /// `pos` must be within `0..=total_len` and strictly greater than the last
    /// added position. In typical usage, `pos` is the index *immediately after*
    /// a detected line terminator.
    pub fn add_line(&mut self, pos: usize) {
        assert!(pos <= self.total_len);
        assert!(self.line_map.last().is_none_or(|&last| pos > last));
        self.line_map.push(pos);
    }

    /// Finishes building and returns the [`LineMap`].
    #[inline]
    pub fn finish(self) -> LineMap {
        LineMap {
            total_len: self.total_len,
            line_map: self.line_map,
        }
    }
}

/// A trait for representing source snippets.
///
/// A snippet is a sequence of *source units* (implementor-defined). Units are
/// typically bytes or characters, but can be anything as long as you can:
/// - map unit positions to line/column via a [`LineMap`], and
/// - map unit positions within a line to columns in a rendered string.
///
/// This trait is the abstraction consumed by [`Annotations`](crate::Annotations):
/// all annotation spans are ranges of unit positions in the snippet.
///
/// # Units and source positions
///
/// All "positions" used by this crate are **0-based indices** in the unit
/// sequence.
///
/// - For [`Utf8Snippet`], units are bytes (byte offsets).
/// - For [`Latin1Snippet`], units are bytes (byte offsets).
/// - For custom snippets, units can be whatever is most convenient (e.g. an
///   index into a token array), as long as [`LineMap`] and
///   [`get_line()`](Snippet::get_line) agree.
pub trait Snippet {
    /// Returns the line mapping for the snippet.
    fn line_map(&self) -> &LineMap;

    /// Returns the [`SnippetLine`] for the given line index.
    ///
    /// `line_i` is a 0-based line index in `0..self.line_map().num_lines()`.
    ///
    /// # Panics
    ///
    /// Panics if `line_i` is out of bounds.
    fn get_line(&self, line_i: usize) -> SnippetLine;
}

/// A single rendered source line with metadata used for rendering snippets
/// with annotations.
///
/// This type is to be constructed in [`Snippet::get_line()`], using
/// [`SnippetLine::builder()`] as starting point.
pub struct SnippetLine {
    pub(crate) text: String,
    metas: Vec<UnitMeta>,
    large_widths: Vec<(usize, usize)>,
    large_utf8_lens: Vec<(usize, usize)>,
}

impl SnippetLine {
    /// Creates a builder for constructing a [`SnippetLine`].
    ///
    /// This is mainly useful when implementing [`Snippet::get_line()`].
    pub fn builder() -> SnippetLineBuilder {
        SnippetLineBuilder {
            text: String::new(),
            metas: Vec::new(),
            large_widths: Vec::new(),
            large_utf8_lens: Vec::new(),
        }
    }

    pub(crate) fn calc_start_col(&self, mut src_pos: usize) -> (usize, usize) {
        while self.metas.get(src_pos).is_some_and(|m| m.is_extra()) {
            src_pos -= 1;
        }

        let width = self.calc_width(src_pos);
        let utf8_len = self.calc_utf8_len(src_pos);

        (width, utf8_len)
    }

    pub(crate) fn calc_end_col(&self, mut src_pos: usize) -> (usize, usize) {
        while self.metas.get(src_pos).is_some_and(|m| m.is_extra())
            && self.metas.get(src_pos + 1).is_some_and(|m| m.is_extra())
        {
            src_pos += 1;
        }

        let width = self.calc_width(src_pos);
        let utf8_len = self.calc_utf8_len(src_pos);

        (width, utf8_len)
    }

    fn calc_utf8_len(&self, orig_len: usize) -> usize {
        let mut utf8_len = 0;
        for (i, meta) in self.metas[..orig_len].iter().enumerate() {
            let len_i = meta.utf8_len();
            if len_i == 0x7F {
                let large_i = self
                    .large_utf8_lens
                    .binary_search_by_key(&i, |&(j, _)| j)
                    .unwrap();
                utf8_len += self.large_utf8_lens[large_i].1;
            } else {
                utf8_len += usize::from(len_i);
            }
        }
        utf8_len
    }

    fn calc_width(&self, orig_len: usize) -> usize {
        let mut width = 0;
        for (i, meta) in self.metas[..orig_len].iter().enumerate() {
            let width_i = meta.width();
            if width_i == 0x7F {
                let large_i = self
                    .large_widths
                    .binary_search_by_key(&i, |&(j, _)| j)
                    .unwrap();
                width += self.large_widths[large_i].1;
            } else {
                width += usize::from(width_i);
            }
        }
        width
    }

    pub(crate) fn gather_styles(&self) -> Vec<(usize, bool)> {
        let mut styles = Vec::with_capacity(self.text.len());

        for (i, meta) in self.metas.iter().enumerate() {
            let meta_len = meta.utf8_len();
            let utf8_len = if meta_len == 0x7F {
                let large_i = self
                    .large_utf8_lens
                    .binary_search_by_key(&i, |&(j, _)| j)
                    .unwrap();
                self.large_utf8_lens[large_i].1
            } else {
                usize::from(meta_len)
            };

            styles.extend(core::iter::repeat_n((usize::MAX, meta.is_alt()), utf8_len));
        }

        styles
    }
}

/// Incremental builder for [`SnippetLine`].
///
/// This builder is intended to be used while scanning a source line and
/// emitting a (possibly transformed) rendered representation.
///
/// If your input may contain control/invisible characters (including tabs),
/// handle them explicitly via [`SnippetLineBuilder::maybe_push_control_char()`]
/// and only use [`SnippetLineBuilder::push_str()`],
/// [`SnippetLineBuilder::push_char()`], and [`SnippetLineBuilder::push_fmt()`]
/// for already-sanitized text.
///
/// Every `push_*` method takes an `orig_len` parameter: the number of *source
/// units* consumed by what you are pushing. This preserves the mapping from
/// positions in the original source (bytes/chars/units) to columns in the
/// rendered line.
pub struct SnippetLineBuilder {
    text: String,
    metas: Vec<UnitMeta>,
    large_widths: Vec<(usize, usize)>,
    large_utf8_lens: Vec<(usize, usize)>,
}

impl SnippetLineBuilder {
    /// Finishes the line and returns a [`SnippetLine`].
    ///
    /// `orig_eol_len` is the number of source units used by the line terminator
    /// (for example, `0` for the last line, `1` for `"\n"` or `"\r"`, `2` for
    /// `"\r\n"`).
    pub fn finish(mut self, orig_eol_len: usize) -> SnippetLine {
        self.push_meta(orig_eol_len, 1, 0, false);
        SnippetLine {
            text: self.text,
            metas: self.metas,
            large_widths: self.large_widths,
            large_utf8_lens: self.large_utf8_lens,
        }
    }

    /// Pushes nothing, consuming `orig_len` source units.
    ///
    /// This is useful for "removing" characters from the rendered output while
    /// keeping the source-to-rendered mapping.
    pub fn push_empty(&mut self, orig_len: usize) {
        self.push_meta(orig_len, 0, 0, false);
    }

    /// Pushes a string fragment.
    ///
    /// `text` should not contain control/invisible characters; if it might,
    /// scan it and use [`SnippetLineBuilder::maybe_push_control_char()`]
    /// (and/or other transformations) instead.
    ///
    /// `orig_len` is the number of source units represented by `text`.
    /// `alt` controls whether the fragment uses the alternate text style.
    pub fn push_str(&mut self, text: &str, orig_len: usize, alt: bool) {
        self.text.push_str(text);
        let width = string_width(text);
        self.push_meta(orig_len, width, text.len(), alt);
    }

    /// Pushes a single character.
    ///
    /// `chr` should not be a control/invisible character. If it might be, call
    /// [`SnippetLineBuilder::maybe_push_control_char()`] first.
    ///
    /// `orig_len` is the number of source units represented by this character
    /// (for UTF-8 sources, callers typically use `chr.len_utf8()`).
    pub fn push_char(&mut self, chr: char, orig_len: usize, alt: bool) {
        let old_text_len = self.text.len();
        self.text.push(chr);
        let new_text_len = self.text.len();

        let width = string_width(&self.text[old_text_len..new_text_len]);
        self.push_meta(orig_len, width, chr.len_utf8(), alt);
    }

    /// Pushes `width` spaces.
    ///
    /// `orig_len` is the number of source units represented by these spaces.
    pub fn push_spaces(&mut self, width: usize, orig_len: usize, alt: bool) {
        let spaces = "                ";
        let mut rem = width;
        while rem != 0 {
            let n = rem.min(spaces.len());
            self.text.push_str(&spaces[..n]);
            rem -= n;
        }
        self.push_meta(orig_len, width, width, alt);
    }

    /// Pushes formatted text.
    ///
    /// The formatted output should not contain control/invisible characters.
    /// If you need to handle such characters, format into a temporary buffer
    /// and scan/push characters using
    /// [`SnippetLineBuilder::maybe_push_control_char()`] as appropriate.
    ///
    /// `orig_len` is the number of source units represented by the formatted
    /// text.
    pub fn push_fmt(&mut self, fmt: core::fmt::Arguments<'_>, orig_len: usize, alt: bool) {
        let old_text_len = self.text.len();
        core::fmt::write(&mut self.text, fmt)
            .expect("a format implementation returned an error unexpectedly");
        let new_text_len = self.text.len();

        let added_len = new_text_len - old_text_len;
        let width = string_width(&self.text[old_text_len..]);

        self.push_meta(orig_len, width, added_len, alt);
    }

    /// Pushes a visible representation of certain control/invisible characters.
    ///
    /// This ensures that characters that are typically invisible (or can affect
    /// layout) are rendered in a safe, explicit way.
    ///
    /// If `chr` matches one of the handled cases, this method pushes a replacement
    /// representation (which may be empty) and returns `true`. Otherwise it leaves
    /// the builder unchanged and returns `false`.  The exact replacement rules are
    /// documented on [`ControlCharStyle`].
    ///
    /// `orig_len` is the number of source units represented by these spaces.
    ///
    /// # Example
    ///
    /// ```
    /// # use sourceannot::{ControlCharStyle, SnippetLine};
    /// # let mut builder = SnippetLine::builder();
    /// # let chr = '\t';
    /// // Assume `chr` is `char` from a UTF-8 source
    /// let is_control = builder.maybe_push_control_char(
    ///     chr,
    ///     chr.len_utf8(),
    ///     4,
    ///     ControlCharStyle::Hexadecimal,
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
            // Each element of `self.metas` corresponds to a byte or unit in the
            // original source, so fill with "extras" for multi-unit chunks  (for
            // example, a multi-byte UTF-8 character, a multi-byte invalid UTF-8
            // sequence or a CRLF line break).
            self.metas.push(UnitMeta::extra());
        }
    }
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
                .finish()
        }
    }
}

impl UnitMeta {
    const MAX_WIDTH: u8 = 0x7F;
    const MAX_UTF8_LEN: u8 = 0x7F;
    const EXTRA_MASK: u16 = 0x8000;
    const ALT_MASK: u16 = 0x0080;

    #[inline]
    fn extra() -> Self {
        Self { inner: 0x8000 }
    }

    #[inline]
    fn new(width: u8, utf8_len: u8, alt: bool) -> Self {
        assert!(width <= Self::MAX_WIDTH);
        assert!(utf8_len <= Self::MAX_UTF8_LEN);
        let alt = if alt { Self::ALT_MASK } else { 0x0000 };
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
        (self.inner & 0x7F) as u8
    }

    #[inline]
    fn utf8_len(&self) -> u8 {
        ((self.inner >> 8) & 0x7F) as u8
    }

    #[inline]
    fn is_alt(&self) -> bool {
        self.inner & Self::ALT_MASK != 0
    }
}

fn string_width(s: &str) -> usize {
    s.chars()
        .map(|c| unicode_width::UnicodeWidthChar::width(c).unwrap_or(1))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::LineMap;

    #[test]
    fn test_line_map() {
        let src = b"abc\ndef\r\nghi\rjkl";

        let line_map = LineMap::with_bytes(src, true);
        assert_eq!(line_map.total_len, src.len());
        assert_eq!(line_map.line_map, [4, 9, 13]);

        let line_map = LineMap::with_bytes(src, false);
        assert_eq!(line_map.total_len, src.len());
        assert_eq!(line_map.line_map, [4, 9]);
    }
}
