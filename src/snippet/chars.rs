use super::{ControlCharStyle, LineMap};

/// A [`Snippet`](super::Snippet) backed by a slice of [`char`].
///
/// Source positions (and annotation ranges) are **indices** into the original
/// `&[char]` slice.
pub struct CharsSnippet<'a> {
    source: &'a [char],
    line_map: LineMap,
    tab_width: usize,
    control_char_style: ControlCharStyle,
    control_char_alt: bool,
}

impl<'a> CharsSnippet<'a> {
    /// Creates a `char`-based source snippet.
    ///
    /// `"\n"` and `"\r\n"` are unconditionally recognized as line breaks. A bare
    /// `"\r"` is treated as a line break only when `cr_is_eol` is `true`.
    ///
    /// Tabs are expanded to `tab_width` spaces. Other control characters are
    /// rendered according to `control_char_style` (see [`ControlCharStyle`])
    /// and can be marked as alternate text via `control_char_alt`.
    pub fn new(
        source: &'a [char],
        cr_is_eol: bool,
        tab_width: usize,
        control_char_style: ControlCharStyle,
        control_char_alt: bool,
    ) -> Self {
        let line_map = LineMap::with_chars(source, cr_is_eol);
        Self {
            source,
            line_map,
            tab_width,
            control_char_style,
            control_char_alt,
        }
    }
}

impl super::Snippet for CharsSnippet<'_> {
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
            if line_source.ends_with(&['\r', '\n']) {
                let new_len = line_source.len() - 2;
                line_source = &line_source[..new_len];
                eol_len = 2;
            } else {
                let (eol, without_eol) = line_source.split_last().unwrap();
                debug_assert!(*eol == '\n' || *eol == '\r');
                line_source = without_eol;
                eol_len = 1;
            }
        }

        let mut line_builder = super::SnippetLine::builder();
        for &chr in line_source {
            let is_control = line_builder.maybe_push_control_char(
                chr,
                1,
                self.tab_width,
                self.control_char_style,
                self.control_char_alt,
            );
            if !is_control {
                line_builder.push_char(chr, 1, false);
            }
        }

        line_builder.finish(eol_len)
    }
}
