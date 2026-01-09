use alloc::string::String;
use alloc::vec::Vec;

use super::{Snippet, SnippetLine, UnitMeta};
use crate::range_set::RangeSet;

mod latin1;
mod utf8;

struct SourceSnippetBuilder {
    start_line: usize,
    lines: Vec<SnippetLine>,
    line_map: Vec<usize>,
    metas: Vec<UnitMeta>,
    large_widths: Vec<(usize, usize)>,
    large_utf8_lens: Vec<(usize, usize)>,
    current_line_text: String,
    current_line_alts: RangeSet<usize>,
}

impl SourceSnippetBuilder {
    fn new(start_line: usize) -> Self {
        Self {
            start_line,
            lines: Vec::new(),
            line_map: Vec::new(),
            metas: Vec::new(),
            large_widths: Vec::new(),
            large_utf8_lens: Vec::new(),
            current_line_text: String::new(),
            current_line_alts: RangeSet::new(),
        }
    }

    fn finish(mut self) -> Snippet {
        self.lines.push(SnippetLine {
            text: self.current_line_text.into_boxed_str(),
            alts: self.current_line_alts,
        });

        Snippet {
            start_line: self.start_line,
            lines: self.lines,
            line_map: self.line_map,
            metas: self.metas,
            large_widths: self.large_widths,
            large_utf8_lens: self.large_utf8_lens,
        }
    }

    fn next_line(&mut self, orig_len: usize) {
        self.lines.push(SnippetLine {
            text: core::mem::take(&mut self.current_line_text).into_boxed_str(),
            alts: core::mem::take(&mut self.current_line_alts),
        });
        if orig_len != 0 {
            self.push_meta(orig_len, 1, 0);
        }
        self.line_map.push(self.metas.len());
    }

    fn push_text(&mut self, text: &str, orig_len: usize, alt: bool) {
        let old_line_len = self.current_line_text.len();
        self.current_line_text.push_str(text);
        let new_line_len = self.current_line_text.len();

        if alt && !text.is_empty() {
            self.current_line_alts
                .insert(old_line_len..=(new_line_len - 1));
        }

        let width = unicode_width::UnicodeWidthStr::width(text);

        self.push_meta(orig_len, width, text.len());
    }

    fn push_char(&mut self, chr: char, width: usize, orig_len: usize, alt: bool) {
        let old_line_len = self.current_line_text.len();
        self.current_line_text.push(chr);
        let new_line_len = self.current_line_text.len();

        if alt {
            self.current_line_alts
                .insert(old_line_len..=(new_line_len - 1));
        }

        self.push_meta(orig_len, width, chr.len_utf8());
    }

    fn push_meta(&mut self, orig_len: usize, width: usize, utf8_len: usize) {
        assert_ne!(orig_len, 0);
        let meta_width = if width >= 0x7F {
            self.large_widths.push((self.metas.len(), width));
            0x7F
        } else {
            width as u8
        };
        let meta_utf8_len = if utf8_len >= 0x7F {
            self.large_utf8_lens.push((self.metas.len(), utf8_len));
            0x7F
        } else {
            utf8_len as u8
        };
        self.metas.push(UnitMeta::new(meta_width, meta_utf8_len));
        for _ in 1..orig_len {
            // Each element of `self.metas` corresponds to a byte or unit in the
            // original source, so fill with "extras" for multi-unit chunks  (for
            // example, a multi-byte UTF-8 character, a multi-byte invalid UTF-8
            // sequence or a CRLF line break).
            self.metas.push(UnitMeta::extra());
        }
    }
}
