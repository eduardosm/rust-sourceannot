use super::utils::{char_orig_len, handle_control_char};
use super::{ControlCharStyle, InvalidSeqStyle, Snippet, char_should_be_replaced};

impl Snippet {
    /// Creates a [`Snippet`] from a UTF-16 (possibly invalid) source.
    ///
    /// # Source units and spans
    ///
    /// The *source unit* for this builder is a **16-bit word** of the original
    /// `source`. Any annotation span you pass later (a `Range<usize>`) is
    /// interpreted as word offsets into this original source sequence.
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
    /// - Any other control character up to U+FFFF, and C0 controls when
    ///   `control_char_style` is [`ControlCharStyle::Hexadecimal`], are
    ///   represented with the hexadecimal value the UTF-16 word, in angle
    ///   brackets, with four digits (`<XXXX>`).
    /// - Any other control character is represented with the hexadecimal
    ///   value of their code point, in angle brackets, with at least four digits
    ///   (`<U+XXXX>`).
    ///
    /// Control characters are rendered as alternate text when `control_char_alt` is
    /// `true`, with the exception of tabs, which are never marked as alternate text.
    ///
    /// # Invalid UTF-16
    ///
    /// - When `invalid_seq_style` is [`InvalidSeqStyle::Replacement`], each
    ///   unpaired surrogate word is replaced with the Unicode Replacement Character
    ///   (U+FFFD, `�`).
    /// - When `invalid_seq_style` is [`InvalidSeqStyle::Hexadecimal`], each
    ///   unpaired surrogate word is represented with its hexadecimal value, in
    ///   angle brackets, with four digits (`<XXXX>`).
    ///
    /// If `invalid_seq_alt` is `true`, the replacement fragments are marked as
    /// "alternate" text.
    pub fn with_utf16_words<I>(
        start_line: usize,
        source: I,
        tab_width: usize,
        control_char_style: ControlCharStyle,
        control_char_alt: bool,
        invalid_seq_style: InvalidSeqStyle,
        invalid_seq_alt: bool,
    ) -> Self
    where
        I: IntoIterator<Item = u16>,
    {
        let mut builder = Snippet::builder(start_line);

        let mut dec_iter = char::decode_utf16(source);
        if let Some(mut cur_dec) = dec_iter.next() {
            loop {
                let mut next_dec = dec_iter.next();
                match cur_dec {
                    Ok(cur_chr) => {
                        if cur_chr == '\r' && next_dec == Some(Ok('\n')) {
                            builder.next_line(2);
                            next_dec = dec_iter.next();
                        } else if cur_chr == '\n' {
                            builder.next_line(1);
                        } else if cur_chr == '\t' {
                            builder.push_spaces(tab_width, 1, false);
                        } else if char_should_be_replaced(cur_chr) {
                            handle_control_char::<16>(
                                &mut builder,
                                cur_chr,
                                control_char_style,
                                control_char_alt,
                            );
                        } else {
                            let orig_len = char_orig_len::<16>(cur_chr);
                            builder.push_char(cur_chr, orig_len, false);
                        }
                    }
                    Err(e) => match invalid_seq_style {
                        InvalidSeqStyle::Replacement => {
                            builder.push_char('\u{FFFD}', 1, invalid_seq_alt);
                        }
                        InvalidSeqStyle::Hexadecimal => {
                            let word = e.unpaired_surrogate();
                            builder.push_fmt(format_args!("<{word:04X}>"), 1, invalid_seq_alt);
                        }
                    },
                }

                cur_dec = match next_dec {
                    Some(dec) => dec,
                    None => break,
                };
            }
        }

        builder.finish()
    }
}
