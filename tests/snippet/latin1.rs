use sourceannot::ControlCharStyle;

use super::test_render_simple;

#[test]
fn test_simple() {
    // 1
    let source = b"123\n456";
    let snippet =
        sourceannot::Snippet::with_latin1(0, source, 4, ControlCharStyle::Replacement, true);

    test_render_simple(&snippet, 0..1, "123", "^  ", "ttt");
    test_render_simple(&snippet, 1..2, "123", " ^ ", "ttt");
    test_render_simple(&snippet, 2..3, "123", "  ^", "ttt");
    test_render_simple(&snippet, 3..4, "123", "   ^", "ttt");
    test_render_simple(&snippet, 4..5, "456", "^  ", "ttt");
    test_render_simple(&snippet, 5..6, "456", " ^ ", "ttt");
    test_render_simple(&snippet, 6..7, "456", "  ^", "ttt");
    test_render_simple(&snippet, 7..7, "456", "   ^", "ttt");

    // 2
    let source = b"123\n456\n";
    let snippet =
        sourceannot::Snippet::with_latin1(0, source, 4, ControlCharStyle::Replacement, true);

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
    let source = b"123\n4\xFF6";
    let snippet =
        sourceannot::Snippet::with_latin1(0, source, 4, ControlCharStyle::Replacement, true);

    test_render_simple(&snippet, 4..5, "4\u{FF}6", "^  ", "ttt");
    test_render_simple(&snippet, 5..6, "4\u{FF}6", " ^ ", "ttt");
    test_render_simple(&snippet, 6..7, "4\u{FF}6", "  ^", "ttt");
}

#[test]
fn test_tab() {
    let source = b"123\n4\t6";

    // width 3
    let snippet =
        sourceannot::Snippet::with_latin1(0, source, 3, ControlCharStyle::Replacement, true);

    test_render_simple(&snippet, 4..5, "4   6", "^    ", "ttttt");
    test_render_simple(&snippet, 5..6, "4   6", " ^^^ ", "ttttt");
    test_render_simple(&snippet, 6..7, "4   6", "    ^", "ttttt");

    // width 4
    let snippet =
        sourceannot::Snippet::with_latin1(0, source, 4, ControlCharStyle::Replacement, true);

    test_render_simple(&snippet, 4..5, "4    6", "^     ", "tttttt");
    test_render_simple(&snippet, 5..6, "4    6", " ^^^^ ", "tttttt");
    test_render_simple(&snippet, 6..7, "4    6", "     ^", "tttttt");

    // width 0
    let snippet =
        sourceannot::Snippet::with_latin1(0, source, 0, ControlCharStyle::Replacement, true);

    test_render_simple(&snippet, 4..5, "46", "^ ", "tt");
    test_render_simple(&snippet, 5..6, "46", " ^", "tt");
    test_render_simple(&snippet, 6..7, "46", " ^", "tt");
}

#[test]
fn test_line_breaks() {
    let source = b"123\r\n4\r6\r\n";
    let snippet =
        sourceannot::Snippet::with_latin1(0, source, 4, ControlCharStyle::Hexadecimal, true);

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
    let source = b"123\n4\x006";

    // alt
    let snippet =
        sourceannot::Snippet::with_latin1(0, source, 4, ControlCharStyle::Replacement, true);

    test_render_simple(&snippet, 4..5, "4\u{2400}6", "^  ", "tTt");
    test_render_simple(&snippet, 5..6, "4\u{2400}6", " ^ ", "tTt");
    test_render_simple(&snippet, 6..7, "4\u{2400}6", "  ^", "tTt");

    // non-alt
    let snippet =
        sourceannot::Snippet::with_latin1(0, source, 4, ControlCharStyle::Replacement, false);

    test_render_simple(&snippet, 4..5, "4\u{2400}6", "^  ", "ttt");
    test_render_simple(&snippet, 5..6, "4\u{2400}6", " ^ ", "ttt");
    test_render_simple(&snippet, 6..7, "4\u{2400}6", "  ^", "ttt");
}

#[test]
fn test_control_char_hex() {
    let source = b"123\n4\x006";

    // alt
    let snippet =
        sourceannot::Snippet::with_latin1(0, source, 4, ControlCharStyle::Hexadecimal, true);

    test_render_simple(&snippet, 4..5, "4<00>6", "^     ", "tTTTTt");
    test_render_simple(&snippet, 5..6, "4<00>6", " ^^^^ ", "tTTTTt");
    test_render_simple(&snippet, 6..7, "4<00>6", "     ^", "tTTTTt");

    // non-alt
    let snippet =
        sourceannot::Snippet::with_latin1(0, source, 4, ControlCharStyle::Hexadecimal, false);

    test_render_simple(&snippet, 4..5, "4<00>6", "^     ", "tttttt");
    test_render_simple(&snippet, 5..6, "4<00>6", " ^^^^ ", "tttttt");
    test_render_simple(&snippet, 6..7, "4<00>6", "     ^", "tttttt");
}
