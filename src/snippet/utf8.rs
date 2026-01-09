use super::{ControlCharStyle, LineMap};

/// Style for rendering invalid UTF-8 sequences.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum InvalidUtf8SeqStyle {
    /// Render invalid UTF-8 as the replacement character U+FFFD (`�`).
    ///
    /// Note that source positions for this snippet are **byte offsets** into
    /// the original slice. When an invalid sequence spans multiple bytes, the
    /// rendered output will contain a single `�`, but that glyph still
    /// corresponds to (and "consumes") the full number of invalid bytes in the
    /// original source for annotation purposes.
    Replacement,
    /// Render invalid UTF-8 as hexadecimal byte values, each byte as `<XX>`.
    ///
    /// This is useful when you want the invalid bytes to be visible and
    /// "countable" in the rendered output.
    Hexadecimal,
}

/// A [`Snippet`](super::Snippet) backed by a byte slice that is interpreted as
/// UTF-8, optionally containing invalid byte sequences.
///
/// This snippet is designed for diagnostics: it renders normally-valid UTF-8
/// text, but it also renders invalid sequences in a deterministic way (see
/// [`InvalidUtf8SeqStyle`]) so that you can still display and annotate the
/// original input.
///
/// Source positions (and annotation ranges) are **byte offsets** into the
/// original UTF-8 sequence.
pub struct Utf8Snippet<'a> {
    source: &'a [u8],
    line_map: LineMap,
    tab_width: usize,
    control_char_style: ControlCharStyle,
    control_char_alt: bool,
    invalid_seq_style: InvalidUtf8SeqStyle,
    invalid_seq_alt: bool,
}

impl<'a> Utf8Snippet<'a> {
    /// Creates a UTF-8 snippet from a (potentially invalid) byte slice.
    ///
    /// `"\n"` and `"\r\n"` are unconditionally recognized as line breaks. A bare
    /// `"\r"` is treated as a line break only when `cr_is_eol` is `true`.
    ///
    /// Tabs are expanded to `tab_width` spaces. Other control characters are
    /// rendered according to `control_char_style` (see [`ControlCharStyle`])
    /// and can be marked as alternate text via `control_char_alt`.
    ///
    /// Invalid UTF-8 sequences are rendered according to `invalid_seq_style`
    /// (see [`InvalidUtf8SeqStyle`]) and can be marked as alternate text via
    /// `invalid_seq_alt`.
    pub fn new(
        source: &'a [u8],
        cr_is_eol: bool,
        tab_width: usize,
        control_char_style: ControlCharStyle,
        control_char_alt: bool,
        invalid_seq_style: InvalidUtf8SeqStyle,
        invalid_seq_alt: bool,
    ) -> Self {
        let line_map = LineMap::with_slice(source, b'\n', b'\r', cr_is_eol);
        Self {
            source,
            line_map,
            tab_width,
            control_char_style,
            control_char_alt,
            invalid_seq_style,
            invalid_seq_alt,
        }
    }
}

impl super::Snippet for Utf8Snippet<'_> {
    fn line_map(&self) -> &LineMap {
        &self.line_map
    }

    fn get_line(&self, line_i: usize) -> super::SnippetLine {
        let num_lines = self.line_map.num_lines();
        assert!(line_i < num_lines);

        let line_range = self.line_map.line_range(line_i);
        let mut line_source = &self.source[line_range];
        let mut eol_len = 0;
        if line_i + 1 != num_lines {
            if let Some(without_eol) = line_source.strip_suffix(b"\r\n") {
                line_source = without_eol;
                eol_len = 2;
            } else {
                let (&eol, without_eol) = line_source.split_last().unwrap();
                debug_assert!(eol == b'\n' || eol == b'\r');
                line_source = without_eol;
                eol_len = 1;
            }
        }

        let mut line_builder = super::SnippetLine::builder();
        for source_chunk in line_source.utf8_chunks() {
            for chr in source_chunk.valid().chars() {
                let is_control = line_builder.maybe_push_control_char(
                    chr,
                    chr.len_utf8(),
                    self.tab_width,
                    self.control_char_style,
                    self.control_char_alt,
                );
                if !is_control {
                    line_builder.push_char(chr, chr.len_utf8(), false);
                }
            }

            let invalid_utf8 = source_chunk.invalid();
            if !invalid_utf8.is_empty() {
                match self.invalid_seq_style {
                    InvalidUtf8SeqStyle::Replacement => {
                        line_builder.push_char(
                            '\u{FFFD}',
                            invalid_utf8.len(),
                            self.invalid_seq_alt,
                        );
                    }
                    InvalidUtf8SeqStyle::Hexadecimal => {
                        for &byte in invalid_utf8.iter() {
                            line_builder.push_fmt(
                                format_args!("<{byte:02X}>"),
                                1,
                                self.invalid_seq_alt,
                            );
                        }
                    }
                }
            }
        }

        line_builder.finish(eol_len)
    }
}
