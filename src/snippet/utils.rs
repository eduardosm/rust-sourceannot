use super::{ControlCharStyle, SnippetBuilder, char_should_be_replaced};

pub(super) fn build_with_char_iter<const UTF: u8>(
    builder: &mut SnippetBuilder,
    chars: impl IntoIterator<Item = char>,
    tab_width: usize,
    control_char_style: ControlCharStyle,
    control_char_alt: bool,
) {
    let mut chars = chars.into_iter();

    let Some(mut cur_chr) = chars.next() else {
        return;
    };
    loop {
        let mut next_chr = chars.next();
        if cur_chr == '\r' && next_chr == Some('\n') {
            builder.next_line(2);
            next_chr = chars.next();
        } else if cur_chr == '\n' {
            builder.next_line(1);
        } else if cur_chr == '\t' {
            builder.push_spaces(tab_width, 1, false);
        } else if char_should_be_replaced(cur_chr) {
            handle_control_char::<UTF>(builder, cur_chr, control_char_style, control_char_alt);
        } else {
            let orig_len = char_orig_len::<UTF>(cur_chr);
            builder.push_char(cur_chr, orig_len, false);
        }

        cur_chr = match next_chr {
            Some(c) => c,
            None => break,
        };
    }
}

pub(super) fn handle_control_char<const UTF: u8>(
    builder: &mut SnippetBuilder,
    chr: char,
    control_char_style: ControlCharStyle,
    control_char_alt: bool,
) {
    let orig_len = char_orig_len::<UTF>(chr);
    match (chr, control_char_style) {
        ('\u{00}'..='\u{1F}', ControlCharStyle::Replacement) => {
            let replacement = char::try_from(u32::from(chr) + 0x2400).unwrap();
            builder.push_char(replacement, orig_len, control_char_alt);
        }
        ('\u{7F}', ControlCharStyle::Replacement) => {
            builder.push_char('␡', orig_len, control_char_alt);
        }
        ('\u{200D}', _) => {
            // Replace ZERO WIDTH JOINER with nothing
            builder.push_empty(orig_len);
        }
        ('\u{00}'..='\u{FF}', _) if UTF == 0 => {
            // Single-byte character
            builder.push_fmt(
                format_args!("<{:02X}>", u32::from(chr)),
                orig_len,
                control_char_alt,
            );
        }
        ('\u{00}'..='\u{7F}', _) if UTF == 8 => {
            // Single-byte UTF-8 character
            builder.push_fmt(
                format_args!("<{:02X}>", u32::from(chr)),
                orig_len,
                control_char_alt,
            );
        }
        ('\u{0000}'..='\u{FFFF}', _) if UTF == 16 => {
            // Single-word UTF-16 character
            builder.push_fmt(
                format_args!("<{:04X}>", u32::from(chr)),
                orig_len,
                control_char_alt,
            );
        }
        (_, _) => {
            // Other cases are represented as <U+XXXX>
            builder.push_fmt(
                format_args!("<U+{:04X}>", u32::from(chr)),
                orig_len,
                control_char_alt,
            );
        }
    }
}

#[inline]
pub(super) fn char_orig_len<const UTF: u8>(c: char) -> usize {
    match UTF {
        0 => 1,
        8 => c.len_utf8(),
        16 => c.len_utf16(),
        32 => 1,
        _ => unreachable!(),
    }
}
