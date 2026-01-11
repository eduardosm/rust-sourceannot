use super::utils::build_with_char_iter;
use super::{ControlCharStyle, Snippet};

impl Snippet {
    /// Creates a [`Snippet`] from a [`char`] sequence.
    ///
    /// # Source units and spans
    ///
    /// The *source unit* for this builder is a **[`char`]** of the original
    /// `source`. Any annotation span you pass later (a `Range<usize>`) is
    /// interpreted as [`char`]s indices into this original `source`.
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
    /// - Tabs (U+0009) are replaced with `tab_width` spaces and never marked as
    ///   alternate text.
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
    /// # Examples
    ///
    /// If `source` is a [`char`] slice:
    /// ```
    /// # let chars = ['x'];
    /// let snippet = sourceannot::Snippet::with_chars(
    ///     1,
    ///     chars.iter().copied(),
    ///     4,
    ///     sourceannot::ControlCharStyle::Hexadecimal,
    ///     true,
    /// );
    /// ```
    ///
    /// If `source` is a UTF-8 ([`str`]) slice, but you want source units to be
    /// [`char`]s instead of bytes:
    /// ```
    /// # let source = "x";
    /// let snippet = sourceannot::Snippet::with_chars(
    ///     1,
    ///     source.chars(),
    ///     4,
    ///     sourceannot::ControlCharStyle::Hexadecimal,
    ///     true,
    /// );
    /// ```
    pub fn with_chars<I>(
        start_line: usize,
        source: I,
        tab_width: usize,
        control_char_style: ControlCharStyle,
        control_char_alt: bool,
    ) -> Self
    where
        I: IntoIterator<Item = char>,
    {
        let mut builder = Snippet::builder(start_line);
        build_with_char_iter::<32>(
            &mut builder,
            source,
            tab_width,
            control_char_style,
            control_char_alt,
        );
        builder.finish()
    }
}
