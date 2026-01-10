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
    /// Tabs (`\t`) are expanded to `tab_width` spaces. Other control characters
    /// are rendered according to `control_char_style` (see [`ControlCharStyle`]).
    /// If `control_char_alt` is `true`, those replacement fragments are marked as
    /// "alternate" text.
    pub fn with_latin1(
        start_line: usize,
        source: &[u8],
        tab_width: usize,
        control_char_style: ControlCharStyle,
        control_char_alt: bool,
    ) -> Self {
        let mut builder = Snippet::builder(start_line);

        let mut chars = source.iter();
        while let Some(&chr) = chars.next() {
            if chr == b'\r' && chars.as_slice().starts_with(b"\n") {
                builder.next_line(2);
                chars.next().unwrap();
            } else if chr == b'\n' {
                builder.next_line(1);
            } else {
                let is_control = builder.maybe_push_control_char(
                    chr.into(),
                    1,
                    tab_width,
                    control_char_style,
                    control_char_alt,
                );
                if !is_control {
                    builder.push_char(chr.into(), 1, false);
                }
            }
        }

        builder.finish()
    }
}
