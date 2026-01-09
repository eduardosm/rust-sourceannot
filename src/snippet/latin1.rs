use super::{ControlCharStyle, LineMap};

/// A [`SourceSnippet`](super::SourceSnippet) backed by a Latin-1 (ISO-8859-1)
/// byte slice.
///
/// Each input byte is interpreted as the corresponding Unicode scalar value
/// (U+0000 to U+00FF).
///
/// Source positions (and annotation ranges) are **byte offsets** into the
/// original Latin-1 source slice.
pub struct Latin1SourceSnippet<'a> {
    source: &'a [u8],
    line_map: LineMap,
    tab_width: usize,
    control_char_style: ControlCharStyle,
    control_char_alt: bool,
}

impl<'a> Latin1SourceSnippet<'a> {
    /// Creates a Latin-1 source snippet.
    ///
    /// `"\n"` and `"\r\n"` are unconditionally recognized as line breaks. A bare
    /// `"\r"` is treated as a line break only when `cr_is_eol` is `true`.
    ///
    /// Tabs are expanded to `tab_width` spaces. Other control characters are
    /// rendered according to `control_char_style` (see [`ControlCharStyle`])
    /// and can be marked as alternate text via `control_char_alt`.
    pub fn new(
        source: &'a [u8],
        cr_is_eol: bool,
        tab_width: usize,
        control_char_style: ControlCharStyle,
        control_char_alt: bool,
    ) -> Self {
        let line_map = LineMap::with_bytes(source, cr_is_eol);
        Self {
            source,
            line_map,
            tab_width,
            control_char_style,
            control_char_alt,
        }
    }
}

impl super::SourceSnippet for Latin1SourceSnippet<'_> {
    fn line_map(&self) -> &LineMap {
        &self.line_map
    }

    fn get_line(&self, line_i: usize) -> super::SourceSnippetLine {
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

        let mut line_builder = super::SourceSnippetLine::builder();
        for &chr in line_source {
            let is_control = line_builder.maybe_push_control_char(
                chr.into(),
                1,
                self.tab_width,
                self.control_char_style,
                self.control_char_alt,
            );
            if !is_control {
                line_builder.push_char(chr.into(), 1, false);
            }
        }

        line_builder.finish(eol_len)
    }
}
