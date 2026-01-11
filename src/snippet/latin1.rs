use super::utils::build_with_char_iter;
use super::{ControlCharStyle, Snippet};

impl Snippet {
    /// Creates a [`Snippet`] from a Latin-1 (ISO 8859-1) source.
    ///
    /// This builder interprets each byte of `source` as a Unicode scalar value
    /// in the range U+0000 to U+00FF.
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
    /// - Tabs (0x09) are replaced with `tab_width` spaces.
    /// - When `control_char_style` is [`ControlCharStyle::Replacement`], C0
    ///   controls (0x00 to 0x1F, excluding tab) and DEL (0x7F) are replaced
    ///   with their Unicode Control Pictures (␀, ␁, ...).
    /// - Any other control character, and C0 controls when `control_char_style`
    ///   is [`ControlCharStyle::Codepoint`], are represented with the hexadecimal
    ///   value of their code point, in angle brackets, with at least four digits
    ///   (`<U+XXXX>`).
    ///
    /// Control characters are rendered as alternate text when `control_char_alt` is
    /// `true`, with the exception of tabs, which are never marked as alternate text.
    pub fn with_latin1(
        start_line: usize,
        source: &[u8],
        tab_width: usize,
        control_char_style: ControlCharStyle,
        control_char_alt: bool,
    ) -> Self {
        let mut builder = Snippet::builder(start_line);
        build_with_char_iter::<0>(
            &mut builder,
            source.iter().copied().map(char::from),
            tab_width,
            control_char_style,
            control_char_alt,
        );
        builder.finish()
    }
}
