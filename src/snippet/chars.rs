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
    /// Tabs (`\t`) are expanded to `tab_width` spaces. Other control characters
    /// are rendered according to `control_char_style` (see [`ControlCharStyle`]).
    /// If `control_char_alt` is `true`, those replacement fragments are marked as
    /// "alternate" text.
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
        let mut snippet = Snippet::builder(start_line);

        let mut chars = source.into_iter();
        let mut last_is_cr = false;
        loop {
            let chr = chars.next();
            if last_is_cr {
                if let Some('\n') = chr {
                    snippet.next_line(2);
                    last_is_cr = false;
                    continue;
                } else {
                    // Lone `\r`
                    let chr_len = '\r'.len_utf8();
                    let is_control = snippet.maybe_push_control_char(
                        '\r',
                        chr_len,
                        tab_width,
                        control_char_style,
                        control_char_alt,
                    );
                    assert!(is_control);
                }
            }

            let Some(chr) = chr else {
                break;
            };

            last_is_cr = chr == '\r';
            if last_is_cr {
                // do nothing yet, depends on the next char being `\n`
            } else if chr == '\n' {
                snippet.next_line(1);
            } else {
                let is_control = snippet.maybe_push_control_char(
                    chr,
                    1,
                    tab_width,
                    control_char_style,
                    control_char_alt,
                );
                if !is_control {
                    snippet.push_char(chr, 1, false);
                }
            }
        }

        snippet.finish()
    }
}
