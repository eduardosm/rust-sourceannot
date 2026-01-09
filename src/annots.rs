use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::{vec, vec::Vec};

use crate::snippet::SnippetLine;
use crate::{AnnotStyle, MainStyle, Output, Snippet};

/// A collection of annotations attached to a [`Snippet`].
///
/// # Spans
///
/// Each annotation is a [`Range<usize>`](core::ops::Range) whose indices are
/// positions in the snippet's *source unit* sequence (as defined by the concrete
/// [`Snippet`] implementation). For example, [`Utf8Snippet`](crate::Utf8Snippet)
/// uses byte offsets into the original UTF-8 sequence.
///
/// Spans are half-open: `start` is inclusive, `end` is exclusive.
/// Zero-length spans (`start == end`) are allowed and render as a single caret
/// pointing at that position.
///
/// # Labels
///
/// A label is stored as a list of `(String, M)` fragments. This allows
/// attaching different metadata to different parts of the label (for example,
/// to highlight a keyword inside the label). If you do not need per-fragment
/// metadata, you can use a single-element vector.
///
/// Empty label fragments are allowed; the renderer will simply output nothing
/// for them (just the carets or connector lines of the annotation).
///
/// # Example
///
/// See the [crate-level](crate#example) documentation.
pub struct Annotations<'a, M> {
    snippet: &'a dyn Snippet,
    first_line_no: usize,
    main_style: MainStyle<M>,
    annots: Vec<Annotation<M>>,
    max_pos: usize,
}

struct Annotation<M> {
    span: core::ops::Range<usize>,
    style: AnnotStyle<M>,
    label: Vec<(String, M)>,
}

impl<'a, M> Annotations<'a, M> {
    /// Creates a new annotation collection for `snippet`.
    ///
    /// `first_line_no` controls the line number shown for the snippet's first
    /// line (line index `0`). You usually want to use `1` when the snippet
    /// represents a whole source file.
    pub fn new(snippet: &'a dyn Snippet, first_line_no: usize, main_style: MainStyle<M>) -> Self {
        Self {
            snippet,
            first_line_no,
            main_style,
            annots: Vec::new(),
            max_pos: 0,
        }
    }

    /// Adds an annotation span with the given style and label.
    ///
    /// - `span` is a half-open range (`start` inclusive, `end` exclusive)
    ///   expressed in the snippet's source units (see [`Annotations`] docs).
    /// - `label` is a list of text fragments with associated metadata; these
    ///   fragments are concatenated when rendered.
    ///
    /// # Panics
    ///
    /// This method assumes `span.start <= span.end` and that both ends refer
    /// to valid positions in the underlying snippet. Invalid spans may panic
    /// during rendering.
    pub fn add_annotation(
        &mut self,
        span: core::ops::Range<usize>,
        style: AnnotStyle<M>,
        label: Vec<(String, M)>,
    ) {
        self.max_pos = self.max_pos.max(span.end);
        self.annots.push(Annotation { span, style, label });
    }

    pub fn max_line_no_width(&self) -> usize {
        let (max_line_i, _) = self.snippet.line_map().pos_to_line_col(self.max_pos);
        let max_line_no = max_line_i + self.first_line_no;
        (max_line_no.max(1).ilog10() + 1) as usize
    }

    /// Renders the snippet with the annotations.
    ///
    /// If no annotations have been added, this outputs nothing.
    ///
    /// ## Parameters
    ///
    /// - `max_line_no_width` should be at least
    ///   [`self.max_line_no_width()`](Self::max_line_no_width), but it can be
    ///   greater to align vertically the margin of multiple snippets.
    /// - `max_fill_after_first` and `max_fill_before_last` control how many
    ///   *unannotated* lines are rendered when there is a gap between two
    ///   annotated lines:
    ///
    ///   - If the gap size is greater than `max_fill_after_first +
    ///     max_fill_before_last`, the renderer outputs only the first
    ///     `max_fill_after_first` lines after the previous annotated line, then
    ///     a dotted separator line, then the last `max_fill_before_last` lines
    ///     before the next annotated line.
    ///   - Otherwise, all lines in the gap are rendered.
    pub fn render<O: Output<M>>(
        &self,
        max_line_no_width: usize,
        max_fill_after_first: usize,
        max_fill_before_last: usize,
        out: O,
    ) -> Result<(), O::Error> {
        let pre_proc = self.pre_process();
        pre_proc.render(
            max_line_no_width,
            max_fill_after_first,
            max_fill_before_last,
            out,
        )
    }

    fn pre_process(&'a self) -> PreProcAnnots<'a, M> {
        let mut pre_proc = PreProcAnnots::new(self.snippet, self.first_line_no, &self.main_style);
        for annot in self.annots.iter() {
            pre_proc.add_annotation(annot.span.clone(), &annot.style, &annot.label);
        }
        pre_proc
    }
}

struct PreProcAnnots<'a, M> {
    snippet: &'a dyn Snippet,
    first_line_no: usize,
    main_style: &'a MainStyle<M>,
    annots: Vec<PreProcAnnot<'a, M>>,
    lines: BTreeMap<usize, LineData>,
    num_ml_slots: usize,
}

struct PreProcAnnot<'a, M> {
    style: &'a AnnotStyle<M>,
    start_line: usize,
    start_col: usize,
    end_line: usize,
    end_col: usize,
    label: &'a [(String, M)],
    sl_overlaps: bool,
    ml_slot: usize,
}

struct LineData {
    // "sl" stands for single line
    // "ml" stands for multi line
    snippet: SnippetLine,
    sl_annots: Vec<usize>,
    ml_annots_starts: Vec<usize>,
    ml_annots_ends: Vec<usize>,
    sl_carets: Vec<usize>,
    styles: Vec<(usize, bool)>,
}

impl<'a, M> PreProcAnnots<'a, M> {
    fn new(snippet: &'a dyn Snippet, first_line_no: usize, main_style: &'a MainStyle<M>) -> Self {
        Self {
            snippet,
            first_line_no,
            main_style,
            annots: Vec::new(),
            lines: BTreeMap::new(),
            num_ml_slots: 0,
        }
    }

    fn add_annotation(
        &mut self,
        src_span: core::ops::Range<usize>,
        style: &'a AnnotStyle<M>,
        label: &'a [(String, M)],
    ) {
        let (start_line, start_col_src) = self.snippet.line_map().pos_to_line_col(src_span.start);

        let start_line_data = self
            .lines
            .entry(start_line)
            .or_insert_with(|| Self::create_line_data(self.snippet, start_line));

        let (start_col, start_col_utf8) = start_line_data.snippet.calc_start_col(start_col_src);

        let end_line;
        let end_col_src;
        if src_span.end > src_span.start {
            let end_col_src_excl;
            (end_line, end_col_src_excl) =
                self.snippet.line_map().pos_to_line_col(src_span.end - 1);
            end_col_src = end_col_src_excl + 1;
        } else {
            // zero length span
            end_line = start_line;
            end_col_src = start_col_src;
        }

        let annot_i = self.annots.len();

        let mut end_col;
        let mut sl_overlaps = false;
        let mut ml_slot = usize::MAX;

        if start_line == end_line {
            // Single line
            let line_data = start_line_data;

            let end_col_utf8;
            (end_col, end_col_utf8) = line_data.snippet.calc_end_col(end_col_src);
            // Render at least one caret
            if end_col == start_col {
                end_col += 1;
            }

            // Check if annotation overlaps with other single line annotations
            for &prev_annot_i in line_data.sl_annots.iter() {
                let other_annot = &mut self.annots[prev_annot_i];
                if start_col.max(other_annot.start_col) < end_col.min(other_annot.end_col) {
                    sl_overlaps = true;
                    other_annot.sl_overlaps = true;
                }
            }

            // Apply caret styles
            if line_data.sl_carets.len() <= start_col {
                line_data.sl_carets.resize(start_col, usize::MAX);
                line_data.sl_carets.resize(end_col, annot_i);
            } else if line_data.sl_carets.len() <= end_col {
                line_data.sl_carets[start_col..]
                    .iter_mut()
                    .for_each(|sl_caret| {
                        if *sl_caret == usize::MAX {
                            *sl_caret = annot_i;
                        }
                    });
                line_data.sl_carets.resize(end_col, annot_i);
            } else {
                line_data.sl_carets[start_col..end_col]
                    .iter_mut()
                    .for_each(|sl_caret| {
                        if *sl_caret == usize::MAX {
                            *sl_caret = annot_i;
                        }
                    });
            }

            // Apply line text styles
            for chr_style in line_data.styles[start_col_utf8..end_col_utf8].iter_mut() {
                if chr_style.0 == usize::MAX {
                    chr_style.0 = annot_i;
                }
            }

            Self::insert_annot_sorted(&self.annots, start_col, annot_i, &mut line_data.sl_annots);
        } else {
            // Multi line
            for chr_style in start_line_data.styles[start_col_utf8..].iter_mut() {
                if chr_style.0 == usize::MAX {
                    chr_style.0 = annot_i;
                }
            }
            Self::insert_annot_sorted(
                &self.annots,
                start_col,
                annot_i,
                &mut start_line_data.ml_annots_starts,
            );

            let end_line_data = self
                .lines
                .entry(end_line)
                .or_insert_with(|| Self::create_line_data(self.snippet, end_line));
            let end_col_utf8;
            (end_col, end_col_utf8) = end_line_data.snippet.calc_end_col(end_col_src);

            for chr_style in end_line_data.styles[..end_col_utf8].iter_mut() {
                if chr_style.0 == usize::MAX {
                    chr_style.0 = annot_i;
                }
            }
            Self::insert_annot_sorted(
                &self.annots,
                start_col,
                annot_i,
                &mut end_line_data.ml_annots_ends,
            );

            let starts_at_col_0 = start_col == 0;

            let mut used_slots = Vec::new();
            for other_annot in self.annots.iter() {
                if other_annot.start_line == other_annot.end_line {
                    continue;
                }
                let other_starts_at_col_0 = other_annot.start_col == 0;
                let line_overlaps = (starts_at_col_0 && other_annot.end_line == start_line)
                    || (other_starts_at_col_0 && other_annot.start_line == end_line)
                    || start_line.max(other_annot.start_line) < end_line.min(other_annot.end_line);

                if line_overlaps {
                    if other_annot.ml_slot >= used_slots.len() {
                        used_slots.resize(other_annot.ml_slot, false);
                        used_slots.push(true);
                    } else {
                        used_slots[other_annot.ml_slot] = true;
                    }
                }
            }

            ml_slot = used_slots
                .iter()
                .position(|used| !used)
                .unwrap_or(used_slots.len());
            self.num_ml_slots = self.num_ml_slots.max(ml_slot + 1);
        }

        self.annots.push(PreProcAnnot {
            style,
            start_line,
            start_col,
            end_line,
            end_col,
            label,
            sl_overlaps,
            ml_slot,
        });
    }

    fn insert_annot_sorted(
        annots: &[PreProcAnnot<'_, M>],
        start_col: usize,
        annot_i: usize,
        dest: &mut Vec<usize>,
    ) {
        let insert_i = dest
            .binary_search_by_key(&(start_col, annot_i), |other_annot_i| {
                let other_annot = &annots[*other_annot_i];
                (other_annot.start_col, *other_annot_i)
            })
            .unwrap_err();
        dest.insert(insert_i, annot_i);
    }

    fn create_line_data(snippet: &'a dyn Snippet, line_i: usize) -> LineData {
        let snippet_line = snippet.get_line(line_i);
        let styles = snippet_line.gather_styles();
        LineData {
            snippet: snippet_line,
            sl_annots: Vec::new(),
            ml_annots_starts: Vec::new(),
            ml_annots_ends: Vec::new(),
            sl_carets: Vec::new(),
            styles,
        }
    }

    fn render<O: Output<M>>(
        &self,
        max_line_no_width: usize,
        max_fill_after_first: usize,
        max_fill_before_last: usize,
        mut out: O,
    ) -> Result<(), O::Error> {
        if self.lines.is_empty() {
            return Ok(());
        }

        // Renders the left margin of a line:
        // with line number:    `123 │ `
        // without line number: `    │ `
        // with dot:            `    · `
        let put_margin = |line_i: Option<usize>, is_dot: bool, out: &mut O| {
            if let Some(ref margin_style) = self.main_style.margin {
                if let Some(line_i) = line_i {
                    let line_no = line_i + self.first_line_no;
                    let line_no_width = (line_no.max(1).ilog10() + 1) as usize;
                    for _ in 0..(max_line_no_width - line_no_width) {
                        out.put_char(' ', &self.main_style.spaces_meta)?;
                    }
                    out.put_fmt(format_args!("{line_no}"), &margin_style.meta)?;
                    out.put_char(' ', &self.main_style.spaces_meta)?;
                } else {
                    for _ in 0..(max_line_no_width + 1) {
                        out.put_char(' ', &self.main_style.spaces_meta)?;
                    }
                }

                if is_dot {
                    out.put_char(margin_style.dot_char, &margin_style.meta)?;
                } else {
                    out.put_char(margin_style.line_char, &margin_style.meta)?;
                }
                out.put_char(' ', &self.main_style.spaces_meta)?;
            }

            Ok(())
        };

        // Renders the text of a line
        let put_line_text = |line: &SnippetLine, styles: &[(usize, bool)], out: &mut O| {
            assert_eq!(styles.len(), line.text.len());
            let mut chr_i = 0;
            while chr_i < line.text.len() {
                let (annot_i, is_alt) = styles[chr_i];
                let len = styles[chr_i..]
                    .iter()
                    .position(|&(a, alt)| (a, alt) != (annot_i, is_alt))
                    .unwrap_or(styles.len() - chr_i);
                let meta = match (annot_i, is_alt) {
                    (usize::MAX, false) => &self.main_style.text_normal_meta,
                    (usize::MAX, true) => &self.main_style.text_alt_meta,
                    (annot_i, false) => &self.annots[annot_i].style.text_normal_meta,
                    (annot_i, true) => &self.annots[annot_i].style.text_alt_meta,
                };
                out.put_str(&line.text[chr_i..(chr_i + len)], meta)?;
                chr_i += len;
            }
            out.put_char('\n', &self.main_style.spaces_meta)?;
            Ok(())
        };

        let put_fill_line_text = |line: &SnippetLine, out: &mut O| {
            // FIXME: handle alts
            out.put_str(&line.text, &self.main_style.text_normal_meta)?;
            out.put_char('\n', &self.main_style.spaces_meta)?;
            Ok(())
        };

        // Renders the slots of a line
        // example: ` ││ `
        let put_slots_simple = |slots: &[Option<&M>], out: &mut O| {
            for slot in slots.iter().rev() {
                if let Some(slot_style) = *slot {
                    out.put_char(self.main_style.vertical_char, slot_style)?;
                } else {
                    out.put_char(' ', &self.main_style.spaces_meta)?;
                }
            }
            if !slots.is_empty() {
                out.put_char(' ', &self.main_style.spaces_meta)?;
            }
            Ok(())
        };

        // example: ` ╭│ `
        let put_slots_with_short_start =
            |slots: &[Option<&M>], is_slot_start: &[bool], out: &mut O| {
                for (i, slot) in slots.iter().enumerate().rev() {
                    if let Some(slot_meta) = *slot {
                        let chr = if is_slot_start[i] {
                            self.main_style.top_vertical_char
                        } else {
                            self.main_style.vertical_char
                        };
                        out.put_char(chr, slot_meta)?;
                    } else {
                        out.put_char(' ', &self.main_style.spaces_meta)?;
                    }
                }
                if !slots.is_empty() {
                    out.put_char(' ', &self.main_style.spaces_meta)?;
                }
                Ok(())
            };

        // example: ` ╭|─`
        let put_slots_with_start =
            |slots: &[Option<&M>], start_slot: usize, start_slot_meta: &M, out: &mut O| {
                for (i, slot) in slots.iter().enumerate().rev() {
                    if let Some(slot_meta) = *slot {
                        out.put_char(self.main_style.vertical_char, slot_meta)?;
                    } else if i == start_slot {
                        out.put_char(self.main_style.top_corner_char, start_slot_meta)?;
                    } else if i < start_slot {
                        out.put_char(self.main_style.horizontal_char, start_slot_meta)?;
                    } else {
                        out.put_char(' ', &self.main_style.spaces_meta)?;
                    }
                }
                out.put_char(self.main_style.horizontal_char, start_slot_meta)?;
                Ok(())
            };

        // example: ` ╰│─`
        let put_slots_with_end =
            |slots: &[Option<&M>], end_slot: usize, end_slot_meta: &M, out: &mut O| {
                for (i, slot) in slots.iter().enumerate().rev() {
                    if let Some(slot_meta) = *slot {
                        out.put_char(self.main_style.vertical_char, slot_meta)?;
                    } else if i == end_slot {
                        out.put_char(self.main_style.bottom_corner_char, end_slot_meta)?;
                    } else if i < end_slot {
                        out.put_char(self.main_style.horizontal_char, end_slot_meta)?;
                    } else {
                        out.put_char(' ', &self.main_style.spaces_meta)?;
                    }
                }
                out.put_char(self.main_style.horizontal_char, end_slot_meta)?;
                Ok(())
            };

        // example: ` │ │`
        let put_sl_verticals = |sl_annots: &[usize], out: &mut O| {
            let mut col_cursor = 0;
            for &prev_annot_i in sl_annots.iter() {
                let start_col = self.annots[prev_annot_i].start_col;
                if start_col < col_cursor {
                    continue;
                }
                if start_col - col_cursor >= 1 {
                    out.put_char(' ', &self.main_style.spaces_meta)?;
                }
                out.put_char(
                    self.main_style.vertical_char,
                    &self.annots[prev_annot_i].style.line_meta,
                )?;
                col_cursor = start_col + 1;
            }
            Ok(col_cursor)
        };

        let mut ml_slots = vec![None; self.num_ml_slots];
        let mut is_slot_start = vec![false; ml_slots.len()];

        let mut prev_line_i = None;
        for (&line_i, line_data) in self.lines.iter() {
            // Handle lines between annotated lines
            if let Some(prev_line_i) = prev_line_i {
                if (line_i - prev_line_i - 1) > (max_fill_after_first + max_fill_before_last) {
                    for i in 0..max_fill_after_first {
                        let line_i = prev_line_i + 1 + i;
                        let line = self.snippet.get_line(line_i);
                        put_margin(Some(line_i), false, &mut out)?;
                        put_slots_simple(&ml_slots, &mut out)?;
                        put_fill_line_text(&line, &mut out)?;
                    }
                    put_margin(None, true, &mut out)?;
                    put_slots_simple(&ml_slots, &mut out)?;
                    out.put_char('\n', &self.main_style.spaces_meta)?;
                    for i in (0..max_fill_before_last).rev() {
                        let line_i = line_i - 1 - i;
                        let line = self.snippet.get_line(line_i);
                        put_margin(Some(line_i), false, &mut out)?;
                        put_slots_simple(&ml_slots, &mut out)?;
                        put_fill_line_text(&line, &mut out)?;
                    }
                } else {
                    for line_i in (prev_line_i + 1)..line_i {
                        let line = self.snippet.get_line(line_i);
                        put_margin(Some(line_i), false, &mut out)?;
                        put_slots_simple(&ml_slots, &mut out)?;
                        put_fill_line_text(&line, &mut out)?;
                    }
                }
            }

            // Handle multi line annotations that start at the beginning of the line
            for &annot_i in line_data.ml_annots_starts.iter() {
                let annot = &self.annots[annot_i];
                if annot.start_col != 0 {
                    continue;
                }

                assert!(ml_slots[annot.ml_slot].is_none());
                assert!(!is_slot_start[annot.ml_slot]);
                ml_slots[annot.ml_slot] = Some(&annot.style.line_meta);
                is_slot_start[annot.ml_slot] = true;
            }

            put_margin(Some(line_i), false, &mut out)?;
            put_slots_with_short_start(&ml_slots, &is_slot_start, &mut out)?;
            put_line_text(&line_data.snippet, &line_data.styles, &mut out)?;

            is_slot_start.fill(false);

            let last_has_vertical = line_data
                .sl_annots
                .last()
                .is_some_and(|&annot_i| self.annots[annot_i].sl_overlaps);

            // Handle single line annotations
            if !line_data.sl_annots.is_empty() {
                put_margin(None, false, &mut out)?;
                put_slots_simple(&ml_slots, &mut out)?;

                let mut i = 0;
                while i < line_data.sl_carets.len() {
                    let annot_i = line_data.sl_carets[i];
                    let len = line_data.sl_carets[i..]
                        .iter()
                        .position(|&a| a != annot_i)
                        .unwrap_or(line_data.sl_carets.len() - i);
                    let chr = if annot_i == usize::MAX {
                        ' '
                    } else {
                        self.annots[annot_i].style.caret
                    };
                    let style = if annot_i == usize::MAX {
                        &self.main_style.spaces_meta
                    } else {
                        &self.annots[annot_i].style.line_meta
                    };
                    for _ in 0..len {
                        out.put_char(chr, style)?;
                    }
                    i += len;
                }
                if !last_has_vertical {
                    let last_annot = &self.annots[*line_data.sl_annots.last().unwrap()];
                    if last_annot.label.iter().any(|(s, _)| !s.is_empty()) {
                        out.put_char(' ', &self.main_style.spaces_meta)?;
                        for (s, meta) in last_annot.label.iter() {
                            out.put_str(s, meta)?;
                        }
                    }
                }

                out.put_char('\n', &self.main_style.spaces_meta)?;
            }

            let with_verticals = if last_has_vertical || line_data.sl_annots.is_empty() {
                line_data.sl_annots.as_slice()
            } else {
                &line_data.sl_annots[..(line_data.sl_annots.len() - 1)]
            };

            if !with_verticals.is_empty() {
                put_margin(None, false, &mut out)?;
                put_slots_simple(&ml_slots, &mut out)?;
                put_sl_verticals(with_verticals, &mut out)?;
                out.put_char('\n', &self.main_style.spaces_meta)?;
            }

            for (i, &annot_i) in with_verticals.iter().enumerate().rev() {
                put_margin(None, false, &mut out)?;
                put_slots_simple(&ml_slots, &mut out)?;
                let col_cursor = put_sl_verticals(&with_verticals[..i], &mut out)?;
                let start_col = self.annots[annot_i].start_col;
                if col_cursor < start_col {
                    for _ in 0..(start_col - col_cursor) {
                        out.put_char(' ', &self.main_style.spaces_meta)?;
                    }
                }
                for (s, meta) in self.annots[annot_i].label.iter() {
                    out.put_str(s, meta)?;
                }
                out.put_char('\n', &self.main_style.spaces_meta)?;
            }

            // Handle multi line annotations that end at this line
            for &annot_i in line_data.ml_annots_ends.iter() {
                let annot = &self.annots[annot_i];

                assert!(ml_slots[annot.ml_slot].is_some());
                ml_slots[annot.ml_slot] = None;

                put_margin(None, false, &mut out)?;
                put_slots_with_end(&ml_slots, annot.ml_slot, &annot.style.line_meta, &mut out)?;

                if annot.end_col != 0 {
                    for _ in 0..(annot.end_col - 1) {
                        out.put_char(self.main_style.horizontal_char, &annot.style.line_meta)?;
                    }
                }
                out.put_char(annot.style.caret, &annot.style.line_meta)?;
                out.put_char(' ', &self.main_style.spaces_meta)?;
                for (s, meta) in annot.label.iter() {
                    out.put_str(s, meta)?;
                }
                out.put_char('\n', &self.main_style.spaces_meta)?;
            }

            // Handle multi line annotations that start at this line
            // (but not at the beginning of the line)
            for &annot_i in line_data.ml_annots_starts.iter() {
                let annot = &self.annots[annot_i];
                if annot.start_col == 0 {
                    continue;
                }

                put_margin(None, false, &mut out)?;
                put_slots_with_start(&ml_slots, annot.ml_slot, &annot.style.line_meta, &mut out)?;

                assert!(ml_slots[annot.ml_slot].is_none());
                ml_slots[annot.ml_slot] = Some(&annot.style.line_meta);

                for _ in 0..annot.start_col {
                    out.put_char(self.main_style.horizontal_char, &annot.style.line_meta)?;
                }
                out.put_char(annot.style.caret, &annot.style.line_meta)?;
                out.put_char('\n', &self.main_style.spaces_meta)?;
            }

            prev_line_i = Some(line_i);
        }

        Ok(())
    }
}
