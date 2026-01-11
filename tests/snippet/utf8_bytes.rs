use sourceannot::{ControlCharStyle, InvalidUtf8SeqStyle};

use super::test_render_simple;

#[test]
fn test_simple() {
    // 1
    let source = "123\n456";
    let snippet = sourceannot::Snippet::with_utf8_bytes(
        0,
        source.as_bytes(),
        4,
        ControlCharStyle::Replacement,
        true,
        InvalidUtf8SeqStyle::Replacement,
        true,
    );

    test_render_simple(&snippet, 0..1, "123", "^  ", "ttt");
    test_render_simple(&snippet, 1..2, "123", " ^ ", "ttt");
    test_render_simple(&snippet, 2..3, "123", "  ^", "ttt");
    test_render_simple(&snippet, 3..4, "123", "   ^", "ttt");
    test_render_simple(&snippet, 4..5, "456", "^  ", "ttt");
    test_render_simple(&snippet, 5..6, "456", " ^ ", "ttt");
    test_render_simple(&snippet, 6..7, "456", "  ^", "ttt");
    test_render_simple(&snippet, 7..7, "456", "   ^", "ttt");

    // 2
    let source = "123\n456\n";
    let snippet = sourceannot::Snippet::with_utf8_bytes(
        0,
        source.as_bytes(),
        4,
        ControlCharStyle::Replacement,
        true,
        InvalidUtf8SeqStyle::Replacement,
        true,
    );

    test_render_simple(&snippet, 0..1, "123", "^  ", "ttt");
    test_render_simple(&snippet, 1..2, "123", " ^ ", "ttt");
    test_render_simple(&snippet, 2..3, "123", "  ^", "ttt");
    test_render_simple(&snippet, 3..4, "123", "   ^", "ttt");
    test_render_simple(&snippet, 4..5, "456", "^  ", "ttt");
    test_render_simple(&snippet, 5..6, "456", " ^ ", "ttt");
    test_render_simple(&snippet, 6..7, "456", "  ^", "ttt");
    test_render_simple(&snippet, 7..8, "456", "   ^", "ttt");
    test_render_simple(&snippet, 8..8, "", "^", "");
}

#[test]
fn test_non_ascii_char() {
    let source = "123\n4\u{FF}6";
    let snippet = sourceannot::Snippet::with_utf8_bytes(
        0,
        source.as_bytes(),
        4,
        ControlCharStyle::Replacement,
        true,
        InvalidUtf8SeqStyle::Replacement,
        true,
    );

    test_render_simple(&snippet, 4..5, "4\u{FF}6", "^  ", "ttt");
    test_render_simple(&snippet, 5..6, "4\u{FF}6", " ^ ", "ttt");
    test_render_simple(&snippet, 6..7, "4\u{FF}6", " ^ ", "ttt");
    test_render_simple(&snippet, 5..7, "4\u{FF}6", " ^ ", "ttt");
    test_render_simple(&snippet, 7..8, "4\u{FF}6", "  ^", "ttt");
}

#[test]
fn test_tab() {
    let source = "123\n4\t6";

    // width 3
    let snippet = sourceannot::Snippet::with_utf8_bytes(
        0,
        source.as_bytes(),
        3,
        ControlCharStyle::Replacement,
        true,
        InvalidUtf8SeqStyle::Replacement,
        true,
    );

    test_render_simple(&snippet, 4..5, "4   6", "^    ", "ttttt");
    test_render_simple(&snippet, 5..6, "4   6", " ^^^ ", "ttttt");
    test_render_simple(&snippet, 6..7, "4   6", "    ^", "ttttt");

    // width 4
    let snippet = sourceannot::Snippet::with_utf8_bytes(
        0,
        source.as_bytes(),
        4,
        ControlCharStyle::Replacement,
        true,
        InvalidUtf8SeqStyle::Replacement,
        true,
    );

    test_render_simple(&snippet, 4..5, "4    6", "^     ", "tttttt");
    test_render_simple(&snippet, 5..6, "4    6", " ^^^^ ", "tttttt");
    test_render_simple(&snippet, 6..7, "4    6", "     ^", "tttttt");

    // width 0
    let snippet = sourceannot::Snippet::with_utf8_bytes(
        0,
        source.as_bytes(),
        0,
        ControlCharStyle::Replacement,
        true,
        InvalidUtf8SeqStyle::Replacement,
        true,
    );

    test_render_simple(&snippet, 4..5, "46", "^ ", "tt");
    test_render_simple(&snippet, 5..6, "46", " ^", "tt");
    test_render_simple(&snippet, 6..7, "46", " ^", "tt");
}

#[test]
fn test_line_breaks() {
    let source = "123\r\n4\r6\r\n";
    let snippet = sourceannot::Snippet::with_utf8_bytes(
        0,
        source.as_bytes(),
        4,
        ControlCharStyle::Hexadecimal,
        true,
        InvalidUtf8SeqStyle::Replacement,
        true,
    );

    test_render_simple(&snippet, 0..1, "123", "^  ", "ttt");
    test_render_simple(&snippet, 1..2, "123", " ^ ", "ttt");
    test_render_simple(&snippet, 2..3, "123", "  ^", "ttt");
    test_render_simple(&snippet, 3..4, "123", "   ^", "ttt");
    test_render_simple(&snippet, 4..5, "123", "   ^", "ttt");
    test_render_simple(&snippet, 3..5, "123", "   ^", "ttt");
    test_render_simple(&snippet, 5..6, "4<0D>6", "^     ", "tTTTTt");
    test_render_simple(&snippet, 6..7, "4<0D>6", " ^^^^ ", "tTTTTt");
    test_render_simple(&snippet, 7..8, "4<0D>6", "     ^", "tTTTTt");
    test_render_simple(&snippet, 8..9, "4<0D>6", "      ^", "tTTTTt");
    test_render_simple(&snippet, 9..10, "4<0D>6", "      ^", "tTTTTt");
    test_render_simple(&snippet, 8..10, "4<0D>6", "      ^", "tTTTTt");
    test_render_simple(&snippet, 10..10, "", "^", "");
}

#[test]
fn test_control_char_replacement() {
    let source = "123\n4\u{0}6";

    // alt
    let snippet = sourceannot::Snippet::with_utf8_bytes(
        0,
        source.as_bytes(),
        4,
        ControlCharStyle::Replacement,
        true,
        InvalidUtf8SeqStyle::Replacement,
        true,
    );

    test_render_simple(&snippet, 4..5, "4\u{2400}6", "^  ", "tTt");
    test_render_simple(&snippet, 5..6, "4\u{2400}6", " ^ ", "tTt");
    test_render_simple(&snippet, 6..7, "4\u{2400}6", "  ^", "tTt");

    // non-alt
    let snippet = sourceannot::Snippet::with_utf8_bytes(
        0,
        source.as_bytes(),
        4,
        ControlCharStyle::Replacement,
        false,
        InvalidUtf8SeqStyle::Replacement,
        true,
    );

    test_render_simple(&snippet, 4..5, "4\u{2400}6", "^  ", "ttt");
    test_render_simple(&snippet, 5..6, "4\u{2400}6", " ^ ", "ttt");
    test_render_simple(&snippet, 6..7, "4\u{2400}6", "  ^", "ttt");
}

#[test]
fn test_control_char_hex() {
    let source = "123\n4\u{0}6\n7\u{2066}9";

    // alt
    let snippet = sourceannot::Snippet::with_utf8_bytes(
        0,
        source.as_bytes(),
        4,
        ControlCharStyle::Hexadecimal,
        true,
        InvalidUtf8SeqStyle::Replacement,
        true,
    );

    test_render_simple(&snippet, 4..5, "4<00>6", "^     ", "tTTTTt");
    test_render_simple(&snippet, 5..6, "4<00>6", " ^^^^ ", "tTTTTt");
    test_render_simple(&snippet, 6..7, "4<00>6", "     ^", "tTTTTt");
    test_render_simple(&snippet, 8..9, "7<U+2066>9", "^         ", "tTTTTTTTTt");
    test_render_simple(&snippet, 9..10, "7<U+2066>9", " ^^^^^^^^ ", "tTTTTTTTTt");
    test_render_simple(&snippet, 10..11, "7<U+2066>9", " ^^^^^^^^ ", "tTTTTTTTTt");
    test_render_simple(&snippet, 11..12, "7<U+2066>9", " ^^^^^^^^ ", "tTTTTTTTTt");
    test_render_simple(&snippet, 9..12, "7<U+2066>9", " ^^^^^^^^ ", "tTTTTTTTTt");
    test_render_simple(&snippet, 12..13, "7<U+2066>9", "         ^", "tTTTTTTTTt");
    // non-alt
    let snippet = sourceannot::Snippet::with_utf8_bytes(
        0,
        source.as_bytes(),
        4,
        ControlCharStyle::Hexadecimal,
        false,
        InvalidUtf8SeqStyle::Replacement,
        true,
    );

    test_render_simple(&snippet, 4..5, "4<00>6", "^     ", "tttttt");
    test_render_simple(&snippet, 5..6, "4<00>6", " ^^^^ ", "tttttt");
    test_render_simple(&snippet, 6..7, "4<00>6", "     ^", "tttttt");
    test_render_simple(&snippet, 8..9, "7<U+2066>9", "^         ", "tttttttttt");
    test_render_simple(&snippet, 9..10, "7<U+2066>9", " ^^^^^^^^ ", "tttttttttt");
    test_render_simple(&snippet, 10..11, "7<U+2066>9", " ^^^^^^^^ ", "tttttttttt");
    test_render_simple(&snippet, 11..12, "7<U+2066>9", " ^^^^^^^^ ", "tttttttttt");
    test_render_simple(&snippet, 9..12, "7<U+2066>9", " ^^^^^^^^ ", "tttttttttt");
    test_render_simple(&snippet, 12..13, "7<U+2066>9", "         ^", "tttttttttt");
}

#[test]
fn test_invalid_char_replacement() {
    let source = b"123\n4\xF1\x806";

    // alt
    let snippet = sourceannot::Snippet::with_utf8_bytes(
        0,
        source,
        4,
        ControlCharStyle::Replacement,
        true,
        InvalidUtf8SeqStyle::Replacement,
        true,
    );

    test_render_simple(&snippet, 4..5, "4\u{FFFD}6", "^  ", "tTt");
    test_render_simple(&snippet, 5..6, "4\u{FFFD}6", " ^ ", "tTt");
    test_render_simple(&snippet, 6..7, "4\u{FFFD}6", " ^ ", "tTt");
    test_render_simple(&snippet, 5..7, "4\u{FFFD}6", " ^ ", "tTt");
    test_render_simple(&snippet, 7..8, "4\u{FFFD}6", "  ^", "tTt");

    // non-alt
    let snippet = sourceannot::Snippet::with_utf8_bytes(
        0,
        source,
        4,
        ControlCharStyle::Replacement,
        true,
        InvalidUtf8SeqStyle::Replacement,
        false,
    );

    test_render_simple(&snippet, 4..5, "4\u{FFFD}6", "^  ", "ttt");
    test_render_simple(&snippet, 5..6, "4\u{FFFD}6", " ^ ", "ttt");
    test_render_simple(&snippet, 6..7, "4\u{FFFD}6", " ^ ", "ttt");
    test_render_simple(&snippet, 5..7, "4\u{FFFD}6", " ^ ", "ttt");
    test_render_simple(&snippet, 7..8, "4\u{FFFD}6", "  ^", "ttt");
}

#[test]
fn test_invalid_char_hex() {
    let source = b"123\n4\xF1\x806";

    // alt
    let snippet = sourceannot::Snippet::with_utf8_bytes(
        0,
        source,
        4,
        ControlCharStyle::Replacement,
        true,
        InvalidUtf8SeqStyle::Hexadecimal,
        true,
    );

    test_render_simple(&snippet, 4..5, "4<F1><80>6", "^         ", "tTTTTTTTTt");
    test_render_simple(&snippet, 5..6, "4<F1><80>6", " ^^^^     ", "tTTTTTTTTt");
    test_render_simple(&snippet, 6..7, "4<F1><80>6", "     ^^^^ ", "tTTTTTTTTt");
    test_render_simple(&snippet, 5..7, "4<F1><80>6", " ^^^^^^^^ ", "tTTTTTTTTt");
    test_render_simple(&snippet, 7..8, "4<F1><80>6", "         ^", "tTTTTTTTTt");

    // non-alt
    let snippet = sourceannot::Snippet::with_utf8_bytes(
        0,
        source,
        4,
        ControlCharStyle::Replacement,
        true,
        InvalidUtf8SeqStyle::Hexadecimal,
        false,
    );

    test_render_simple(&snippet, 4..5, "4<F1><80>6", "^         ", "tttttttttt");
    test_render_simple(&snippet, 5..6, "4<F1><80>6", " ^^^^     ", "tttttttttt");
    test_render_simple(&snippet, 6..7, "4<F1><80>6", "     ^^^^ ", "tttttttttt");
    test_render_simple(&snippet, 5..7, "4<F1><80>6", " ^^^^^^^^ ", "tttttttttt");
    test_render_simple(&snippet, 7..8, "4<F1><80>6", "         ^", "tttttttttt");
}

#[test]
fn test_wide_char() {
    let source = "123\n4\u{FF12}6";
    let snippet = sourceannot::Snippet::with_utf8_bytes(
        0,
        source.as_bytes(),
        4,
        ControlCharStyle::Replacement,
        true,
        InvalidUtf8SeqStyle::Replacement,
        true,
    );

    test_render_simple(&snippet, 4..5, "4\u{FF12}6", "^   ", "ttt");
    test_render_simple(&snippet, 5..6, "4\u{FF12}6", " ^^ ", "ttt");
    test_render_simple(&snippet, 6..7, "4\u{FF12}6", " ^^ ", "ttt");
    test_render_simple(&snippet, 7..8, "4\u{FF12}6", " ^^ ", "ttt");
    test_render_simple(&snippet, 5..8, "4\u{FF12}6", " ^^ ", "ttt");
    test_render_simple(&snippet, 8..9, "4\u{FF12}6", "   ^", "ttt");
}
