use super::{SourceLine, SourceSnippet, SourceUnitMeta};
use crate::range_set::RangeSet;

mod utf8;

struct SourceSnippetBuilder {
    start_line: usize,
    lines: Vec<SourceLine>,
    line_map: Vec<usize>,
    metas: Vec<SourceUnitMeta>,
    current_line_text: String,
    current_line_alts: RangeSet<usize>,
    current_line_width: usize,
}

impl SourceSnippetBuilder {
    fn new(start_line: usize) -> Self {
        Self {
            start_line,
            lines: Vec::new(),
            line_map: Vec::new(),
            metas: Vec::new(),
            current_line_text: String::new(),
            current_line_alts: RangeSet::new(),
            current_line_width: 0,
        }
    }

    fn finish(mut self) -> SourceSnippet {
        self.lines.push(SourceLine {
            text: self.current_line_text.into_boxed_str(),
            alts: self.current_line_alts,
            width: self.current_line_width,
        });

        SourceSnippet {
            start_line: self.start_line,
            lines: self.lines,
            line_map: self.line_map,
            metas: self.metas,
        }
    }

    fn next_line(&mut self, extra_widths: &[usize]) {
        self.lines.push(SourceLine {
            text: std::mem::take(&mut self.current_line_text).into_boxed_str(),
            alts: std::mem::take(&mut self.current_line_alts),
            width: std::mem::replace(&mut self.current_line_width, 0),
        });
        self.metas
            .extend(extra_widths.iter().map(|&w| SourceUnitMeta::new(w, 0)));
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
        self.current_line_width += width;

        self.metas.push(SourceUnitMeta::new(width, text.len()));
        for _ in 1..orig_len {
            // Each element of `snippet.widths` corresponds to a byte in `source`,
            // so fill with -1 for multi-unit chunks.
            self.metas.push(SourceUnitMeta::extra());
        }
    }

    fn push_char(&mut self, chr: char, width: usize, orig_len: usize, alt: bool) {
        let old_line_len = self.current_line_text.len();
        self.current_line_text.push(chr);
        let new_line_len = self.current_line_text.len();
        self.current_line_width += width;

        if alt {
            self.current_line_alts
                .insert(old_line_len..=(new_line_len - 1));
        }

        self.metas.push(SourceUnitMeta::new(width, chr.len_utf8()));
        for _ in 1..orig_len {
            // Each element of `snippet.widths` corresponds to a byte in `source`,
            // so fill with -1 for multi-unit characters.
            self.metas.push(SourceUnitMeta::extra());
        }
    }
}
