use super::utils::build_with_char_iter;
use super::{ControlCharStyle, InvalidSeqStyle, Snippet};

impl Snippet {
    /// Creates a [`Snippet`] from a valid UTF-8 source.
    ///
    /// # Source units and spans
    ///
    /// The *source unit* for this builder is a **byte** of the original `source`.
    /// Any annotation span you pass later (a `Range<usize>`) is interpreted as
    /// byte offsets into this original `source` slice.
    ///
    /// If you want the source units to be [`char`]s instead of bytes, use
    /// [`Snippet::with_chars()`], passing [`source.chars()`](str::chars) as
    /// `source`.
    ///
    /// # Line breaks
    ///
    /// - `\n` and `\r\n` are treated as line breaks.
    /// - A lone `\r` is *not* a line break; it is handled like any other control
    ///   character.
    ///
    /// # Control characters
    ///
    /// Control characters are those for which
    /// [`char_should_be_replaced()`](crate::char_should_be_replaced)
    /// returns `true`.
    ///
    /// - Tabs (U+0009) are replaced with `tab_width` spaces.
    /// - ZERO WIDTH JOINER (U+200D) is replaced with nothing (but still accounts
    ///   for its original source unit length).
    /// - When `control_char_style` is [`ControlCharStyle::Replacement`], C0
    ///   controls (U+0000 to U+001F, excluding tab) and DEL (U+007F) are
    ///   replaced with their Unicode Control Pictures (␀, ␁, ...).
    /// - Any other control character, and C0 controls when `control_char_style`
    ///   is [`ControlCharStyle::Hexadecimal`], are represented with the hexadecimal
    ///   value of their code point, in angle brackets, with at least four digits
    ///   (`<U+XXXX>`).
    ///
    /// Control characters are rendered as alternate text when `control_char_alt` is
    /// `true`, with the exception of tabs, which are never marked as alternate text.
    pub fn with_utf8(
        start_line: usize,
        source: &str,
        tab_width: usize,
        control_char_style: ControlCharStyle,
        control_char_alt: bool,
    ) -> Self {
        let mut builder = Snippet::builder(start_line);
        build_with_char_iter::<8>(
            &mut builder,
            source.chars(),
            tab_width,
            control_char_style,
            control_char_alt,
        );
        builder.finish()
    }

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
    /// Control characters are those for which
    /// [`char_should_be_replaced()`](crate::char_should_be_replaced)
    /// returns `true`.
    ///
    /// - Tabs (U+0009) are replaced with `tab_width` spaces.
    /// - ZERO WIDTH JOINER (U+200D) is replaced with nothing (but still accounts
    ///   for its original source unit length).
    /// - When `control_char_style` is [`ControlCharStyle::Replacement`], C0
    ///   controls (U+0000 to U+001F, excluding tab) and DEL (U+007F) are
    ///   replaced with their Unicode Control Pictures (␀, ␁, ...).
    /// - Any other control character, and C0 controls when `control_char_style`
    ///   is [`ControlCharStyle::Hexadecimal`], are represented with the hexadecimal
    ///   value of their code point, in angle brackets, with at least four digits
    ///   (`<U+XXXX>`).
    ///
    /// Control characters are rendered as alternate text when `control_char_alt` is
    /// `true`, with the exception of tabs, which are never marked as alternate text.
    ///
    /// # Invalid UTF-8
    ///
    /// - When `invalid_seq_style` is [`InvalidSeqStyle::Replacement`], each
    ///   invalid UTF-8 sequence is replaced with the Unicode Replacement Character
    ///   (U+FFFD, `�`).
    /// - When `invalid_seq_style` is [`InvalidSeqStyle::Hexadecimal`], each byte
    ///   of an invalid UTF-8 sequence is represented with its hexadecimal value,
    ///   in angle brackets, with two digits (`<XX>`).
    ///
    /// If `invalid_seq_alt` is `true`, the replacement fragments are marked as
    /// "alternate" text.
    pub fn with_utf8_bytes(
        start_line: usize,
        source: &[u8],
        tab_width: usize,
        control_char_style: ControlCharStyle,
        control_char_alt: bool,
        invalid_seq_style: InvalidSeqStyle,
        invalid_seq_alt: bool,
    ) -> Self {
        let mut builder = Snippet::builder(start_line);

        for source_chunk in source.utf8_chunks() {
            build_with_char_iter::<8>(
                &mut builder,
                source_chunk.valid().chars(),
                tab_width,
                control_char_style,
                control_char_alt,
            );

            let invalid_utf8 = source_chunk.invalid();
            if !invalid_utf8.is_empty() {
                match invalid_seq_style {
                    InvalidSeqStyle::Replacement => {
                        builder.push_char('\u{FFFD}', invalid_utf8.len(), invalid_seq_alt);
                    }
                    InvalidSeqStyle::Hexadecimal => {
                        for &byte in invalid_utf8.iter() {
                            builder.push_fmt(format_args!("<{byte:02X}>"), 1, invalid_seq_alt);
                        }
                    }
                }
            }
        }

        builder.finish()
    }
}
