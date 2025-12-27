use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::string::ToString as _;
use alloc::{vec, vec::Vec};

use crate::snippet::SourceSpan;
use crate::{AnnotStyle, MainStyle, SourceSnippet};

/// A collection of annotations for a source snippet.
#[derive(Debug)]
pub struct Annotations<'a, M> {
    snippet: &'a SourceSnippet,
    main_style: MainStyle<M>,
    annots: Vec<AnnotData<M>>,
    lines: BTreeMap<usize, LineData>,
    num_ml_slots: usize,
}

#[derive(Debug)]
struct AnnotData<M> {
    style: AnnotStyle<M>,
    span: SourceSpan,
    label: Vec<(String, M)>,
    sl_overlaps: bool,
    ml_slot: usize,
}

#[derive(Debug)]
struct LineData {
    // "sl" stands for single line
    // "ml" stands for multi line
    sl_annots: Vec<usize>,
    ml_annots_starts: Vec<usize>,
    ml_annots_ends: Vec<usize>,
    sl_carets: Vec<usize>,
    styles: Vec<(usize, bool)>,
}

impl<'a, M> Annotations<'a, M> {
    pub fn new(snippet: &'a SourceSnippet, main_style: MainStyle<M>) -> Self {
        Self {
            snippet,
            main_style,
            annots: Vec::new(),
            lines: BTreeMap::new(),
            num_ml_slots: 0,
        }
    }

    pub fn add_annotation(
        &mut self,
        span: core::ops::Range<usize>,
        style: AnnotStyle<M>,
        label: Vec<(String, M)>,
    ) {
        let mut annot = AnnotData {
            style,
            span: self.snippet.convert_span(span.start, span.end),
            label,
            sl_overlaps: false,
            ml_slot: usize::MAX,
        };
        let annot_i = self.annots.len();

        let line_data = self
            .lines
            .entry(annot.span.start_line)
            .or_insert_with(|| Self::create_line_data(self.snippet, annot.span.start_line));
        if annot.span.start_line == annot.span.end_line {
            // Single line

            // Render one caret for zero length spans
            annot.span.end_col = annot.span.end_col.max(annot.span.start_col + 1);

            // Check if annotation overlaps with other single line annotations
            for &prev_annot_i in line_data.sl_annots.iter() {
                let other_annot = &mut self.annots[prev_annot_i];
                if annot.span.start_col.max(other_annot.span.start_col)
                    < annot.span.end_col.min(other_annot.span.end_col)
                {
                    annot.sl_overlaps = true;
                    other_annot.sl_overlaps = true;
                }
            }

            // Apply caret styles
            if line_data.sl_carets.len() <= annot.span.start_col {
                line_data.sl_carets.resize(annot.span.start_col, usize::MAX);
                line_data.sl_carets.resize(annot.span.end_col, annot_i);
            } else if line_data.sl_carets.len() <= annot.span.end_col {
                line_data.sl_carets[annot.span.start_col..]
                    .iter_mut()
                    .for_each(|sl_caret| {
                        if *sl_caret == usize::MAX {
                            *sl_caret = annot_i;
                        }
                    });
                line_data.sl_carets.resize(annot.span.end_col, annot_i);
            } else {
                line_data.sl_carets[annot.span.start_col..annot.span.end_col]
                    .iter_mut()
                    .for_each(|sl_caret| {
                        if *sl_caret == usize::MAX {
                            *sl_caret = annot_i;
                        }
                    });
            }

            // Apply line text styles
            for chr_style in line_data.styles[annot.span.start_utf8..annot.span.end_utf8].iter_mut()
            {
                if chr_style.0 == usize::MAX {
                    chr_style.0 = annot_i;
                }
            }

            Self::insert_annot_sorted(&self.annots, &annot, annot_i, &mut line_data.sl_annots);
        } else {
            // Multi line
            for chr_style in line_data.styles[annot.span.start_utf8..].iter_mut() {
                if chr_style.0 == usize::MAX {
                    chr_style.0 = annot_i;
                }
            }
            Self::insert_annot_sorted(
                &self.annots,
                &annot,
                annot_i,
                &mut line_data.ml_annots_starts,
            );

            let end_line_data = self
                .lines
                .entry(annot.span.end_line)
                .or_insert_with(|| Self::create_line_data(self.snippet, annot.span.end_line));

            for chr_style in end_line_data.styles[..annot.span.end_utf8].iter_mut() {
                if chr_style.0 == usize::MAX {
                    chr_style.0 = annot_i;
                }
            }
            Self::insert_annot_sorted(
                &self.annots,
                &annot,
                annot_i,
                &mut end_line_data.ml_annots_ends,
            );

            let starts_at_col_0 = annot.span.start_col == 0;

            let mut used_slots = Vec::new();
            for other_annot in self.annots.iter() {
                if other_annot.span.start_line == other_annot.span.end_line {
                    continue;
                }
                let other_starts_at_col_0 = other_annot.span.start_col == 0;
                let line_overlaps = (starts_at_col_0
                    && other_annot.span.end_line == annot.span.start_line)
                    || (other_starts_at_col_0
                        && other_annot.span.start_line == annot.span.end_line)
                    || annot.span.start_line.max(other_annot.span.start_line)
                        < annot.span.end_line.min(other_annot.span.end_line);

                if line_overlaps {
                    if other_annot.ml_slot >= used_slots.len() {
                        used_slots.resize(other_annot.ml_slot, false);
                        used_slots.push(true);
                    } else {
                        used_slots[other_annot.ml_slot] = true;
                    }
                }
            }

            annot.ml_slot = used_slots
                .iter()
                .position(|used| !used)
                .unwrap_or(used_slots.len());
            self.num_ml_slots = self.num_ml_slots.max(annot.ml_slot + 1);
        }

        self.annots.push(annot);
    }

    fn insert_annot_sorted(
        annots: &[AnnotData<M>],
        annot: &AnnotData<M>,
        annot_i: usize,
        dest: &mut Vec<usize>,
    ) {
        let insert_i = dest
            .binary_search_by_key(&(annot.span.start_col, annot_i), |other_annot_i| {
                let other_annot = &annots[*other_annot_i];
                (other_annot.span.start_col, *other_annot_i)
            })
            .unwrap_err();
        dest.insert(insert_i, annot_i);
    }

    fn create_line_data(snippet: &'a SourceSnippet, line_i: usize) -> LineData {
        let snippet_line = snippet.line(line_i);
        let mut styles = vec![(usize::MAX, false); snippet_line.text.len()];
        for alt_range in snippet_line.alts.ranges() {
            styles[alt_range].fill((usize::MAX, true));
        }
        LineData {
            sl_annots: Vec::new(),
            ml_annots_starts: Vec::new(),
            ml_annots_ends: Vec::new(),
            sl_carets: Vec::new(),
            styles,
        }
    }

    pub fn max_line_no_width(&self) -> usize {
        if let Some((max_line_i, _)) = self.lines.last_key_value() {
            let max_line_no = max_line_i + self.snippet.start_line();
            (max_line_no.max(1).ilog10() + 1) as usize
        } else {
            0
        }
    }
}

impl<M: Clone> Annotations<'_, M> {
    /// Renders the snippet with the annotations.
    ///
    /// `max_line_no_width` should be at least
    /// [`self.max_line_no_width()`](Self::max_line_no_width), but
    /// it can be greater to align the margin of multiple snippets.
    pub fn render(
        &self,
        max_line_no_width: usize,
        max_fill_after_first: usize,
        max_fill_before_last: usize,
    ) -> Vec<(String, M)> {
        if self.lines.is_empty() {
            return Vec::new();
        }

        let start_line = self.snippet.start_line();

        let mut parts = Vec::new();

        // Renders the left margin of a line:
        // with line number:    `123 │ `
        // without line number: `    │ `
        // with dot:            `    · `
        let put_margin = |line_i: Option<usize>, is_dot: bool, parts: &mut Vec<(String, M)>| {
            if let Some(ref margin_style) = self.main_style.margin {
                if let Some(line_i) = line_i {
                    let line_no = line_i + start_line;
                    let line_no_str = line_no.to_string();
                    let line_no_width = line_no_str.len();
                    parts.push((line_no_str, margin_style.meta.clone()));
                    parts.push((
                        " ".repeat(max_line_no_width - line_no_width + 1),
                        self.main_style.spaces_meta.clone(),
                    ));
                } else {
                    parts.push((
                        " ".repeat(max_line_no_width + 1),
                        self.main_style.spaces_meta.clone(),
                    ));
                }

                if is_dot {
                    parts.push((margin_style.dot_char.into(), margin_style.meta.clone()));
                } else {
                    parts.push((margin_style.line_char.into(), margin_style.meta.clone()));
                }
                parts.push((' '.into(), self.main_style.spaces_meta.clone()));
            }
        };

        // Renders the text of a line
        let put_line_text =
            |line_i: usize, styles: &[(usize, bool)], parts: &mut Vec<(String, M)>| {
                let line = self.snippet.line(line_i);
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
                    parts.push((String::from(&line.text[chr_i..(chr_i + len)]), meta.clone()));
                    chr_i += len;
                }
                parts.push(('\n'.into(), self.main_style.spaces_meta.clone()));
            };

        let put_fill_line_text = |line_i: usize, parts: &mut Vec<(String, M)>| {
            let line = self.snippet.line(line_i);
            parts.push((
                String::from(&*line.text),
                self.main_style.text_normal_meta.clone(),
            ));
            parts.push(('\n'.into(), self.main_style.spaces_meta.clone()));
        };

        // Renders the slots of a line
        // example: ` ││ `
        let put_slots_simple = |slots: &[Option<&M>], parts: &mut Vec<(String, M)>| {
            for slot in slots.iter().rev() {
                if let Some(slot_style) = *slot {
                    parts.push((self.main_style.vertical_char.into(), slot_style.clone()));
                } else {
                    parts.push((' '.into(), self.main_style.spaces_meta.clone()));
                }
            }
            if !slots.is_empty() {
                parts.push((' '.into(), self.main_style.spaces_meta.clone()));
            }
        };

        // example: ` ╭│ `
        let put_slots_with_short_start =
            |slots: &[Option<&M>], is_slot_start: &[bool], parts: &mut Vec<(String, M)>| {
                for (i, slot) in slots.iter().enumerate().rev() {
                    if let Some(slot_meta) = *slot {
                        let chr = if is_slot_start[i] {
                            self.main_style.top_vertical_char
                        } else {
                            self.main_style.vertical_char
                        };
                        parts.push((chr.into(), slot_meta.clone()));
                    } else {
                        parts.push((' '.into(), self.main_style.spaces_meta.clone()));
                    }
                }
                if !slots.is_empty() {
                    parts.push((' '.into(), self.main_style.spaces_meta.clone()));
                }
            };

        // example: ` ╭|─`
        let put_slots_with_start =
            |slots: &[Option<&M>],
             start_slot: usize,
             start_slot_meta: &M,
             parts: &mut Vec<(String, M)>| {
                for (i, slot) in slots.iter().enumerate().rev() {
                    if let Some(slot_meta) = *slot {
                        parts.push((self.main_style.vertical_char.into(), slot_meta.clone()));
                    } else if i == start_slot {
                        parts.push((
                            self.main_style.top_corner_char.into(),
                            start_slot_meta.clone(),
                        ));
                    } else if i < start_slot {
                        parts.push((
                            self.main_style.horizontal_char.into(),
                            start_slot_meta.clone(),
                        ));
                    } else {
                        parts.push((' '.into(), self.main_style.spaces_meta.clone()));
                    }
                }
                parts.push((
                    self.main_style.horizontal_char.into(),
                    start_slot_meta.clone(),
                ));
            };

        // example: ` ╰│─`
        let put_slots_with_end = |slots: &[Option<&M>],
                                  end_slot: usize,
                                  end_slot_meta: &M,
                                  parts: &mut Vec<(String, M)>| {
            for (i, slot) in slots.iter().enumerate().rev() {
                if let Some(slot_meta) = *slot {
                    parts.push((self.main_style.vertical_char.into(), slot_meta.clone()));
                } else if i == end_slot {
                    parts.push((
                        self.main_style.bottom_corner_char.into(),
                        end_slot_meta.clone(),
                    ));
                } else if i < end_slot {
                    parts.push((
                        self.main_style.horizontal_char.into(),
                        end_slot_meta.clone(),
                    ));
                } else {
                    parts.push((' '.into(), self.main_style.spaces_meta.clone()));
                }
            }
            parts.push((
                self.main_style.horizontal_char.into(),
                end_slot_meta.clone(),
            ));
        };

        // example: ` │ │`
        let put_sl_verticals = |sl_annots: &[usize], parts: &mut Vec<(String, M)>| -> usize {
            let mut col_cursor = 0;
            for &prev_annot_i in sl_annots.iter() {
                let start_col = self.annots[prev_annot_i].span.start_col;
                if start_col < col_cursor {
                    continue;
                }
                if start_col - col_cursor >= 1 {
                    parts.push((
                        " ".repeat(start_col - col_cursor),
                        self.main_style.spaces_meta.clone(),
                    ));
                }
                parts.push((
                    self.main_style.vertical_char.into(),
                    self.annots[prev_annot_i].style.line_meta.clone(),
                ));
                col_cursor = start_col + 1;
            }
            col_cursor
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
                        put_margin(Some(line_i), false, &mut parts);
                        put_slots_simple(&ml_slots, &mut parts);
                        put_fill_line_text(line_i, &mut parts);
                    }
                    put_margin(None, true, &mut parts);
                    put_slots_simple(&ml_slots, &mut parts);
                    parts.push(('\n'.into(), self.main_style.spaces_meta.clone()));
                    for i in (0..max_fill_before_last).rev() {
                        let line_i = line_i - 1 - i;
                        put_margin(Some(line_i), false, &mut parts);
                        put_slots_simple(&ml_slots, &mut parts);
                        put_fill_line_text(line_i, &mut parts);
                    }
                } else {
                    for line_i in (prev_line_i + 1)..line_i {
                        put_margin(Some(line_i), false, &mut parts);
                        put_slots_simple(&ml_slots, &mut parts);
                        put_fill_line_text(line_i, &mut parts);
                    }
                }
            }

            // Handle multi line annotations that start at the beginning of the line
            for &annot_i in line_data.ml_annots_starts.iter() {
                let annot = &self.annots[annot_i];
                if annot.span.start_col != 0 {
                    continue;
                }

                assert!(ml_slots[annot.ml_slot].is_none());
                assert!(!is_slot_start[annot.ml_slot]);
                ml_slots[annot.ml_slot] = Some(&annot.style.line_meta);
                is_slot_start[annot.ml_slot] = true;
            }

            put_margin(Some(line_i), false, &mut parts);
            put_slots_with_short_start(&ml_slots, &is_slot_start, &mut parts);
            put_line_text(line_i, &line_data.styles, &mut parts);

            is_slot_start.fill(false);

            let last_has_vertical = line_data
                .sl_annots
                .last()
                .is_some_and(|&annot_i| self.annots[annot_i].sl_overlaps);

            // Handle single line annotations
            if !line_data.sl_annots.is_empty() {
                put_margin(None, false, &mut parts);
                put_slots_simple(&ml_slots, &mut parts);

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
                        self.main_style.spaces_meta.clone()
                    } else {
                        self.annots[annot_i].style.line_meta.clone()
                    };
                    parts.push((core::iter::repeat_n(chr, len).collect(), style));
                    i += len;
                }
                if !last_has_vertical {
                    let last_annot = &self.annots[*line_data.sl_annots.last().unwrap()];
                    if last_annot.label.iter().any(|(s, _)| !s.is_empty()) {
                        parts.push((' '.into(), self.main_style.spaces_meta.clone()));
                        parts.extend(last_annot.label.iter().cloned());
                    }
                }

                parts.push(('\n'.into(), self.main_style.spaces_meta.clone()));
            }

            let with_verticals = if last_has_vertical || line_data.sl_annots.is_empty() {
                line_data.sl_annots.as_slice()
            } else {
                &line_data.sl_annots[..(line_data.sl_annots.len() - 1)]
            };

            if !with_verticals.is_empty() {
                put_margin(None, false, &mut parts);
                put_slots_simple(&ml_slots, &mut parts);
                put_sl_verticals(with_verticals, &mut parts);
                parts.push(('\n'.into(), self.main_style.spaces_meta.clone()));
            }

            for (i, &annot_i) in with_verticals.iter().enumerate().rev() {
                put_margin(None, false, &mut parts);
                put_slots_simple(&ml_slots, &mut parts);
                let col_cursor = put_sl_verticals(&with_verticals[..i], &mut parts);
                let start_col = self.annots[annot_i].span.start_col;
                if col_cursor < start_col {
                    parts.push((
                        " ".repeat(start_col - col_cursor),
                        self.main_style.spaces_meta.clone(),
                    ));
                }
                parts.extend(self.annots[annot_i].label.iter().cloned());
                parts.push(('\n'.into(), self.main_style.spaces_meta.clone()));
            }

            // Handle multi line annotations that end at this line
            for &annot_i in line_data.ml_annots_ends.iter() {
                let annot = &self.annots[annot_i];

                assert!(ml_slots[annot.ml_slot].is_some());
                ml_slots[annot.ml_slot] = None;

                put_margin(None, false, &mut parts);
                put_slots_with_end(&ml_slots, annot.ml_slot, &annot.style.line_meta, &mut parts);

                if annot.span.end_col != 0 {
                    parts.push((
                        core::iter::repeat_n(
                            self.main_style.horizontal_char,
                            annot.span.end_col - 1,
                        )
                        .collect(),
                        annot.style.line_meta.clone(),
                    ));
                }
                parts.push((annot.style.caret.into(), annot.style.line_meta.clone()));
                parts.push((' '.into(), self.main_style.spaces_meta.clone()));
                parts.extend(annot.label.iter().cloned());
                parts.push(('\n'.into(), self.main_style.spaces_meta.clone()));
            }

            // Handle multi line annotations that start at this line
            // (but not at the beginning of the line)
            for &annot_i in line_data.ml_annots_starts.iter() {
                let annot = &self.annots[annot_i];
                if annot.span.start_col == 0 {
                    continue;
                }

                put_margin(None, false, &mut parts);
                put_slots_with_start(&ml_slots, annot.ml_slot, &annot.style.line_meta, &mut parts);

                assert!(ml_slots[annot.ml_slot].is_none());
                ml_slots[annot.ml_slot] = Some(&annot.style.line_meta);

                parts.push((
                    core::iter::repeat_n(self.main_style.horizontal_char, annot.span.start_col)
                        .collect(),
                    annot.style.line_meta.clone(),
                ));
                parts.push((annot.style.caret.into(), annot.style.line_meta.clone()));
                parts.push(('\n'.into(), self.main_style.spaces_meta.clone()));
            }

            prev_line_i = Some(line_i);
        }

        parts
    }
}
