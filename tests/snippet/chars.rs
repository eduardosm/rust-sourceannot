use sourceannot::{CharsSnippet, ControlCharStyle, Snippet as _};

use super::test_render_simple;

#[test]
fn test_simple() {
    // 1
    let source = ['1', '2', '3', '\n', '4', '5', '6'];
    let snippet = CharsSnippet::new(&source, false, 4, ControlCharStyle::Replacement, true);

    assert_eq!(snippet.line_map().num_lines(), 2);

    test_render_simple(&snippet, 0..1, "123", "^  ", "ttt");
    test_render_simple(&snippet, 1..2, "123", " ^ ", "ttt");
    test_render_simple(&snippet, 2..3, "123", "  ^", "ttt");
    test_render_simple(&snippet, 3..4, "123", "   ^", "ttt");
    test_render_simple(&snippet, 4..5, "456", "^  ", "ttt");
    test_render_simple(&snippet, 5..6, "456", " ^ ", "ttt");
    test_render_simple(&snippet, 6..7, "456", "  ^", "ttt");
    test_render_simple(&snippet, 7..7, "456", "   ^", "ttt");

    // 2
    let source = ['1', '2', '3', '\n', '4', '5', '6', '\n'];
    let snippet = CharsSnippet::new(&source, false, 4, ControlCharStyle::Replacement, true);

    assert_eq!(snippet.line_map().num_lines(), 3);

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
    let source = ['1', '2', '3', '\n', '4', '\u{FF}', '6'];
    let snippet = CharsSnippet::new(&source, false, 4, ControlCharStyle::Replacement, true);

    assert_eq!(snippet.line_map().num_lines(), 2);

    test_render_simple(&snippet, 4..5, "4\u{FF}6", "^  ", "ttt");
    test_render_simple(&snippet, 5..6, "4\u{FF}6", " ^ ", "ttt");
    test_render_simple(&snippet, 6..7, "4\u{FF}6", "  ^", "ttt");
}

#[test]
fn test_tab() {
    let source = ['1', '2', '3', '\n', '4', '\t', '6'];

    // width 3
    let snippet = CharsSnippet::new(&source, false, 3, ControlCharStyle::Replacement, true);

    assert_eq!(snippet.line_map().num_lines(), 2);

    test_render_simple(&snippet, 4..5, "4   6", "^    ", "ttttt");
    test_render_simple(&snippet, 5..6, "4   6", " ^^^ ", "ttttt");
    test_render_simple(&snippet, 6..7, "4   6", "    ^", "ttttt");

    // width 4
    let snippet = CharsSnippet::new(&source, false, 4, ControlCharStyle::Replacement, true);

    assert_eq!(snippet.line_map().num_lines(), 2);

    test_render_simple(&snippet, 4..5, "4    6", "^     ", "tttttt");
    test_render_simple(&snippet, 5..6, "4    6", " ^^^^ ", "tttttt");
    test_render_simple(&snippet, 6..7, "4    6", "     ^", "tttttt");

    // width 0
    let snippet = CharsSnippet::new(&source, false, 0, ControlCharStyle::Replacement, true);

    assert_eq!(snippet.line_map().num_lines(), 2);

    test_render_simple(&snippet, 4..5, "46", "^ ", "tt");
    test_render_simple(&snippet, 5..6, "46", " ^", "tt");
    test_render_simple(&snippet, 6..7, "46", " ^", "tt");
}

#[test]
fn test_line_breaks() {
    let source = ['1', '2', '3', '\r', '\n', '4', '\r', '6', '\r', '\n'];

    // CR is not EOL
    let snippet = CharsSnippet::new(&source, false, 4, ControlCharStyle::Hexadecimal, true);

    assert_eq!(snippet.line_map().num_lines(), 3);

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

    // CR is EOL
    let snippet = CharsSnippet::new(&source, true, 4, ControlCharStyle::Hexadecimal, true);

    assert_eq!(snippet.line_map().num_lines(), 4);

    test_render_simple(&snippet, 0..1, "123", "^  ", "ttt");
    test_render_simple(&snippet, 1..2, "123", " ^ ", "ttt");
    test_render_simple(&snippet, 2..3, "123", "  ^", "ttt");
    test_render_simple(&snippet, 3..4, "123", "   ^", "ttt");
    test_render_simple(&snippet, 4..5, "123", "   ^", "ttt");
    test_render_simple(&snippet, 3..5, "123", "   ^", "ttt");
    test_render_simple(&snippet, 5..6, "4", "^", "t");
    test_render_simple(&snippet, 6..7, "4", " ^", "t");
    test_render_simple(&snippet, 7..8, "6", "^", "t");
    test_render_simple(&snippet, 8..9, "6", " ^", "t");
    test_render_simple(&snippet, 9..10, "6", " ^", "t");
    test_render_simple(&snippet, 8..10, "6", " ^", "t");
    test_render_simple(&snippet, 10..10, "", "^", "");
}

#[test]
fn test_control_char_replacement() {
    let source = ['1', '2', '3', '\n', '4', '\u{0}', '6'];

    // alt
    let snippet = CharsSnippet::new(&source, false, 4, ControlCharStyle::Replacement, true);

    assert_eq!(snippet.line_map().num_lines(), 2);

    test_render_simple(&snippet, 4..5, "4\u{2400}6", "^  ", "tTt");
    test_render_simple(&snippet, 5..6, "4\u{2400}6", " ^ ", "tTt");
    test_render_simple(&snippet, 6..7, "4\u{2400}6", "  ^", "tTt");

    // non-alt
    let snippet = CharsSnippet::new(&source, false, 4, ControlCharStyle::Replacement, false);

    assert_eq!(snippet.line_map().num_lines(), 2);

    test_render_simple(&snippet, 4..5, "4\u{2400}6", "^  ", "ttt");
    test_render_simple(&snippet, 5..6, "4\u{2400}6", " ^ ", "ttt");
    test_render_simple(&snippet, 6..7, "4\u{2400}6", "  ^", "ttt");
}

#[test]
fn test_control_char_hex() {
    let source = [
        '1', '2', '3', '\n', '4', '\u{0}', '6', '\n', '7', '\u{2066}', '9',
    ];

    // alt
    let snippet = CharsSnippet::new(&source, false, 4, ControlCharStyle::Hexadecimal, true);

    assert_eq!(snippet.line_map().num_lines(), 3);

    test_render_simple(&snippet, 4..5, "4<00>6", "^     ", "tTTTTt");
    test_render_simple(&snippet, 5..6, "4<00>6", " ^^^^ ", "tTTTTt");
    test_render_simple(&snippet, 6..7, "4<00>6", "     ^", "tTTTTt");
    test_render_simple(&snippet, 8..9, "7<2066>9", "^       ", "tTTTTTTt");
    test_render_simple(&snippet, 9..10, "7<2066>9", " ^^^^^^ ", "tTTTTTTt");
    test_render_simple(&snippet, 10..11, "7<2066>9", "       ^", "tTTTTTTt");

    // non-alt
    let snippet = CharsSnippet::new(&source, false, 4, ControlCharStyle::Hexadecimal, false);

    assert_eq!(snippet.line_map().num_lines(), 3);

    test_render_simple(&snippet, 4..5, "4<00>6", "^     ", "tttttt");
    test_render_simple(&snippet, 5..6, "4<00>6", " ^^^^ ", "tttttt");
    test_render_simple(&snippet, 6..7, "4<00>6", "     ^", "tttttt");
    test_render_simple(&snippet, 8..9, "7<2066>9", "^       ", "tttttttt");
    test_render_simple(&snippet, 9..10, "7<2066>9", " ^^^^^^ ", "tttttttt");
    test_render_simple(&snippet, 10..11, "7<2066>9", "       ^", "tttttttt");
}

#[test]
fn test_wide_char() {
    let source = ['1', '2', '3', '\n', '4', '\u{FF12}', '6'];
    let snippet = CharsSnippet::new(&source, false, 4, ControlCharStyle::Replacement, true);

    assert_eq!(snippet.line_map().num_lines(), 2);

    test_render_simple(&snippet, 4..5, "4\u{FF12}6", "^   ", "ttt");
    test_render_simple(&snippet, 5..6, "4\u{FF12}6", " ^^ ", "ttt");
    test_render_simple(&snippet, 6..7, "4\u{FF12}6", "   ^", "ttt");
}
