#![warn(
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_qualifications
)]
#![forbid(unsafe_code)]

// Rendering tests focused on rendering itself and span handling.

use sourceannot::{AnnotStyle, Annotations, MainStyle, MarginStyle, Snippet};

const MAIN_STYLE: MainStyle<char> = MainStyle {
    margin: Some(MarginStyle {
        line_char: '│',
        discontinuity_chars: [' ', ' ', '·'],
        meta: 'm',
    }),
    horizontal_char: '─',
    vertical_char: '│',
    top_vertical_char: '╭',
    top_corner_char: '╭',
    bottom_corner_char: '╰',
    spaces_meta: 's',
    text_normal_meta: 't',
    text_alt_meta: 'T',
};

const ANNOT_STYLE_1: AnnotStyle<char> = AnnotStyle {
    caret: '^',
    text_normal_meta: 'a',
    text_alt_meta: 'A',
    line_meta: 'l',
};

const ANNOT_STYLE_2: AnnotStyle<char> = AnnotStyle {
    caret: '-',
    text_normal_meta: 'b',
    text_alt_meta: 'B',
    line_meta: 'L',
};

struct TestOutput {
    text: String,
    styles: String,
}

impl sourceannot::Output<char> for &mut TestOutput {
    type Error = core::convert::Infallible;

    fn put_str(&mut self, s: &str, &style: &char) -> Result<(), Self::Error> {
        self.text.push_str(s);
        for c in s.chars() {
            self.styles.push(style);
            if c == '\n' {
                self.styles.push('\n');
            }
        }

        Ok(())
    }
}

#[track_caller]
fn test_render(
    annots: &Annotations<'_, char>,
    max_fill_after_first: usize,
    max_fill_before_last: usize,
    expected_text: &str,
    expected_styles: &str,
) {
    let mut output = TestOutput {
        text: String::new(),
        styles: String::new(),
    };
    let max_line_no_width = annots.max_line_no_width();
    annots.render(
        max_line_no_width,
        max_fill_after_first,
        max_fill_before_last,
        &mut output,
    );

    assert_eq!(output.text, expected_text);
    assert_eq!(output.styles, expected_styles);
}

#[test]
fn test_render_single_line() {
    // "1234\n5678\n90ab\ncdef\n"
    let lines = ["1234", "5678", "90ab", "cdef"];
    let mut builder = Snippet::builder(1);
    for line in &lines {
        for c in line.chars() {
            builder.push_char(c, 1, false);
        }
        builder.next_line(1);
    }
    let snippet = builder.finish();

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(1..4, &ANNOT_STYLE_1, vec![("test".into(), '1')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 1234
              │  ^^^ test
        "},
        indoc::indoc! {"
            msmstaaas
            ssmssllls1111s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(1..4, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(10..12, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 1234
              │  ^^^ test 1
              ·
            3 │ 90ab
              │ -- test 2
        "},
        indoc::indoc! {"
            msmstaaas
            ssmssllls111111s
            mmms
            msmsbbtts
            ssmsLLs222222s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(0..2, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(3..4, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 1234
              │ ^^ - test 2
              │ │
              │ test 1
        "},
        indoc::indoc! {"
            msmsaatbs
            ssmsllsLs222222s
            ssmsls
            ssms111111s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(1..2, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(0..4, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 1234
              │ -^--
              │ ││
              │ │test 1
              │ test 2
        "},
        indoc::indoc! {"
            msmsbabbs
            ssmsLlLLs
            ssmsLls
            ssmsL111111s
            ssms222222s
        "},
    );
}

#[test]
fn test_render_multi_line() {
    // "1234\n5678\n90ab\ncdef\n"
    let lines = ["1234", "5678", "90ab", "cdef"];
    let mut builder = Snippet::builder(1);
    for line in &lines {
        for c in line.chars() {
            builder.push_char(c, 1, false);
        }
        builder.next_line(1);
    }
    let snippet = builder.finish();

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(1..12, &ANNOT_STYLE_1, vec![("test".into(), '1')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │   1234
              │ ╭──^
              · │
            3 │ │ 90ab
              │ ╰──^ test
        "},
        indoc::indoc! {"
            msmssstaaas
            ssmslllls
            mmmsls
            msmslsaatts
            ssmslllls1111s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(1..12, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(1..12, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │    1234
              │  ╭──^
              │ ╭│──-
              · ││
            3 │ ││ 90ab
              │ │╰──^ test 1
              │ ╰───- test 2
        "},
        indoc::indoc! {"
            msmsssstaaas
            ssmsslllls
            ssmsLlLLLs
            mmmsLls
            msmsLlsaatts
            ssmsLlllls111111s
            ssmsLLLLLs222222s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(0..11, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(6..18, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │  ╭ 1234
            2 │  │ 5678
              │ ╭│──-
            3 │ ││ 90ab
              │ │╰─^ test 1
            4 │ │  cdef
              │ ╰────- test 2
        "},
        indoc::indoc! {"
            msmsslsaaaas
            msmsslstbbbs
            ssmsLlLLLs
            msmsLlsattts
            ssmsLllls111111s
            msmsLssbbbts
            ssmsLLLLLLs222222s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(6..18, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(0..11, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ ╭  1234
            2 │ │  5678
              │ │╭──^
            3 │ ││ 90ab
              │ ╰│─- test 2
            4 │  │ cdef
              │  ╰───^ test 1
        "},
        indoc::indoc! {"
            msmsLssbbbbs
            msmsLsstaaas
            ssmsLlllls
            msmsLlsbttts
            ssmsLlLLs222222s
            msmsslsaaats
            ssmssllllls111111s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(0..18, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(6..11, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │  ╭ 1234
            2 │  │ 5678
              │ ╭│──-
            3 │ ││ 90ab
              │ ╰│─- test 2
            4 │  │ cdef
              │  ╰───^ test 1
        "},
        indoc::indoc! {"
            msmsslsaaaas
            msmsslstbbbs
            ssmsLlLLLs
            msmsLlsbttts
            ssmsLlLLs222222s
            msmsslsaaats
            ssmssllllls111111s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(0..7, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(6..18, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ ╭ 1234
            2 │ │ 5678
              │ ╰──^ test 1
              │ ╭──-
              · │
            4 │ │ cdef
              │ ╰───- test 2
        "},
        indoc::indoc! {"
            msmslsaaaas
            msmslsaabbs
            ssmslllls111111s
            ssmsLLLLs
            mmmsLs
            msmsLsbbbts
            ssmsLLLLLs222222s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(6..18, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(0..7, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ ╭ 1234
            2 │ │ 5678
              │ ╰──- test 2
              │ ╭──^
              · │
            4 │ │ cdef
              │ ╰───^ test 1
        "},
        indoc::indoc! {"
            msmsLsbbbbs
            msmsLsbaaas
            ssmsLLLLs222222s
            ssmslllls
            mmmsls
            msmslsaaats
            ssmsllllls111111s
        "},
    );
}

#[test]
fn test_render_multi_line_wide_break() {
    // "1234\r\n5678\r\n90ab\r\ncdef\r\n"
    let lines = ["1234", "5678", "90ab", "cdef"];
    let mut builder = Snippet::builder(1);
    for line in &lines {
        for c in line.chars() {
            builder.push_char(c, 1, false);
        }
        builder.next_line(2); // two-unit line break (CRLF)
    }
    let snippet = builder.finish();

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(1..14, &ANNOT_STYLE_1, vec![("test".into(), '1')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │   1234
              │ ╭──^
              · │
            3 │ │ 90ab
              │ ╰──^ test
        "},
        indoc::indoc! {"
            msmssstaaas
            ssmslllls
            mmmsls
            msmslsaatts
            ssmslllls1111s
        "},
    );
}

#[test]
fn test_render_mixed_single_line_and_multi_line() {
    // "1234\n5678\n90ab\ncdef\n"
    let lines = ["1234", "5678", "90ab", "cdef"];
    let mut builder = Snippet::builder(1);
    for line in &lines {
        for c in line.chars() {
            builder.push_char(c, 1, false);
        }
        builder.next_line(1);
    }
    let snippet = builder.finish();

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(1..12, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(6..9, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │   1234
              │ ╭──^
            2 │ │ 5678
              │ │  --- test 2
            3 │ │ 90ab
              │ ╰──^ test 1
        "},
        indoc::indoc! {"
            msmssstaaas
            ssmslllls
            msmslstbbbs
            ssmslssLLLs222222s
            msmslsaatts
            ssmslllls111111s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(1..12, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(0..4, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │   1234
              │   ---- test 2
              │ ╭──^
              · │
            3 │ │ 90ab
              │ ╰──^ test 1
        "},
        indoc::indoc! {"
            msmsssbaaas
            ssmsssLLLLs222222s
            ssmslllls
            mmmsls
            msmslsaatts
            ssmslllls111111s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(0..4, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(1..12, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │   1234
              │   ^^^^ test 1
              │ ╭──-
              · │
            3 │ │ 90ab
              │ ╰──- test 2
        "},
        indoc::indoc! {"
            msmsssaaaas
            ssmssslllls111111s
            ssmsLLLLs
            mmmsLs
            msmsLsbbtts
            ssmsLLLLs222222s
        "},
    );
}

#[test]
fn test_render_zero_len_span() {
    // "1234\n5678\n90ab\ncdef\n"
    let lines = ["1234", "5678", "90ab", "cdef"];
    let mut builder = Snippet::builder(1);
    for line in &lines {
        for c in line.chars() {
            builder.push_char(c, 1, false);
        }
        builder.next_line(1);
    }
    let snippet = builder.finish();

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(0..0, &ANNOT_STYLE_1, vec![("test".into(), '1')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 1234
              │ ^ test
        "},
        indoc::indoc! {"
            msmstttts
            ssmsls1111s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(1..1, &ANNOT_STYLE_1, vec![("test".into(), '1')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 1234
              │  ^ test
        "},
        indoc::indoc! {"
            msmstttts
            ssmssls1111s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(2..2, &ANNOT_STYLE_1, vec![("test".into(), '1')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 1234
              │   ^ test
        "},
        indoc::indoc! {"
            msmstttts
            ssmsssls1111s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(3..3, &ANNOT_STYLE_1, vec![("test".into(), '1')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 1234
              │    ^ test
        "},
        indoc::indoc! {"
            msmstttts
            ssmssssls1111s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(4..4, &ANNOT_STYLE_1, vec![("test".into(), '1')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 1234
              │     ^ test
        "},
        indoc::indoc! {"
            msmstttts
            ssmsssssls1111s
        "},
    );
}

#[test]
fn test_render_multi_char_unit() {
    // "12345\n6abc7", where "abc" is a single source unit
    let mut builder = Snippet::builder(1);
    for c in "12345".chars() {
        builder.push_char(c, 1, false);
    }
    builder.next_line(1);
    builder.push_char('6', 1, false);
    builder.push_str("abc", 1, false);
    builder.push_char('7', 1, false);
    let snippet = builder.finish();

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(1..4, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(7..8, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 12345
              │  ^^^ test 1
            2 │ 6abc7
              │  --- test 2
        "},
        indoc::indoc! {"
            msmstaaats
            ssmssllls111111s
            msmstbbbts
            ssmssLLLs222222s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(1..4, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(6..8, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 12345
              │  ^^^ test 1
            2 │ 6abc7
              │ ---- test 2
        "},
        indoc::indoc! {"
            msmstaaats
            ssmssllls111111s
            msmsbbbbts
            ssmsLLLLs222222s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(1..4, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(7..9, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 12345
              │  ^^^ test 1
            2 │ 6abc7
              │  ---- test 2
        "},
        indoc::indoc! {"
            msmstaaats
            ssmssllls111111s
            msmstbbbbs
            ssmssLLLLs222222s
        "},
    );
}

#[test]
fn test_render_multi_unit_char() {
    // "123\n678", where '7' is a three-unit character.
    let mut builder = Snippet::builder(1);
    for c in "123".chars() {
        builder.push_char(c, 1, false);
    }
    builder.next_line(1);
    builder.push_char('6', 1, false);
    builder.push_char('7', 3, false);
    builder.push_char('8', 1, false);
    let snippet = builder.finish();

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(1..2, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(5..6, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 123
              │  ^ test 1
            2 │ 678
              │  - test 2
        "},
        indoc::indoc! {"
            msmstats
            ssmssls111111s
            msmstbts
            ssmssLs222222s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(1..2, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(6..7, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 123
              │  ^ test 1
            2 │ 678
              │  - test 2
        "},
        indoc::indoc! {"
            msmstats
            ssmssls111111s
            msmstbts
            ssmssLs222222s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(1..2, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(7..8, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 123
              │  ^ test 1
            2 │ 678
              │  - test 2
        "},
        indoc::indoc! {"
            msmstats
            ssmssls111111s
            msmstbts
            ssmssLs222222s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(1..2, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(5..8, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 123
              │  ^ test 1
            2 │ 678
              │  - test 2
        "},
        indoc::indoc! {"
            msmstats
            ssmssls111111s
            msmstbts
            ssmssLs222222s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(1..2, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(4..5, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 123
              │  ^ test 1
            2 │ 678
              │ - test 2
        "},
        indoc::indoc! {"
            msmstats
            ssmssls111111s
            msmsbtts
            ssmsLs222222s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(1..2, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(4..6, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 123
              │  ^ test 1
            2 │ 678
              │ -- test 2
        "},
        indoc::indoc! {"
            msmstats
            ssmssls111111s
            msmsbbts
            ssmsLLs222222s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(1..2, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(4..7, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 123
              │  ^ test 1
            2 │ 678
              │ -- test 2
        "},
        indoc::indoc! {"
            msmstats
            ssmssls111111s
            msmsbbts
            ssmsLLs222222s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(1..2, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(4..8, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 123
              │  ^ test 1
            2 │ 678
              │ -- test 2
        "},
        indoc::indoc! {"
            msmstats
            ssmssls111111s
            msmsbbts
            ssmsLLs222222s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(1..2, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(5..9, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 123
              │  ^ test 1
            2 │ 678
              │  -- test 2
        "},
        indoc::indoc! {"
            msmstats
            ssmssls111111s
            msmstbbs
            ssmssLLs222222s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(1..2, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(6..9, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 123
              │  ^ test 1
            2 │ 678
              │  -- test 2
        "},
        indoc::indoc! {"
            msmstats
            ssmssls111111s
            msmstbbs
            ssmssLLs222222s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(1..2, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(7..9, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 123
              │  ^ test 1
            2 │ 678
              │  -- test 2
        "},
        indoc::indoc! {"
            msmstats
            ssmssls111111s
            msmstbbs
            ssmssLLs222222s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(1..2, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(8..9, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 123
              │  ^ test 1
            2 │ 678
              │   - test 2
        "},
        indoc::indoc! {"
            msmstats
            ssmssls111111s
            msmsttbs
            ssmsssLs222222s
        "},
    );
}

#[test]
fn test_render_textless_unit() {
    // "1234\n5678", with a textless unit between '6' and '7'
    let mut builder = Snippet::builder(1);
    for c in "1234".chars() {
        builder.push_char(c, 1, false);
    }
    builder.next_line(1);
    builder.push_char('5', 1, false);
    builder.push_char('6', 1, false);
    builder.push_empty(1);
    builder.push_char('7', 1, false);
    builder.push_char('8', 1, false);
    let snippet = builder.finish();

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(1..3, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(6..7, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 1234
              │  ^^ test 1
            2 │ 5678
              │  - test 2
        "},
        indoc::indoc! {"
            msmstaats
            ssmsslls111111s
            msmstbtts
            ssmssLs222222s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(1..3, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(6..8, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 1234
              │  ^^ test 1
            2 │ 5678
              │  -- test 2
        "},
        indoc::indoc! {"
            msmstaats
            ssmsslls111111s
            msmstbtts
            ssmssLLs222222s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(1..3, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(6..9, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 1234
              │  ^^ test 1
            2 │ 5678
              │  -- test 2
        "},
        indoc::indoc! {"
            msmstaats
            ssmsslls111111s
            msmstbbts
            ssmssLLs222222s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(1..3, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(7..8, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 1234
              │  ^^ test 1
            2 │ 5678
              │   - test 2
        "},
        indoc::indoc! {"
            msmstaats
            ssmsslls111111s
            msmstttts
            ssmsssLs222222s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(1..3, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(8..9, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 1234
              │  ^^ test 1
            2 │ 5678
              │   - test 2
        "},
        indoc::indoc! {"
            msmstaats
            ssmsslls111111s
            msmsttbts
            ssmsssLs222222s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(1..3, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(8..10, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 1234
              │  ^^ test 1
            2 │ 5678
              │   -- test 2
        "},
        indoc::indoc! {"
            msmstaats
            ssmsslls111111s
            msmsttbbs
            ssmsssLLs222222s
        "},
    );
}

#[test]
fn test_render_line_break_1() {
    // "1234\n5678\n90ab\ncdef\n"
    let lines = ["1234", "5678", "90ab", "cdef"];
    let mut builder = Snippet::builder(1);
    for line in &lines {
        for c in line.chars() {
            builder.push_char(c, 1, false);
        }
        builder.next_line(1);
    }
    let snippet = builder.finish();

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(4..4, &ANNOT_STYLE_1, vec![("test".into(), '1')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 1234
              │     ^ test
        "},
        indoc::indoc! {"
            msmstttts
            ssmsssssls1111s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(4..5, &ANNOT_STYLE_1, vec![("test".into(), '1')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 1234
              │     ^ test
        "},
        indoc::indoc! {"
            msmstttts
            ssmsssssls1111s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(3..4, &ANNOT_STYLE_1, vec![("test".into(), '1')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 1234
              │    ^ test
        "},
        indoc::indoc! {"
            msmstttas
            ssmssssls1111s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(3..5, &ANNOT_STYLE_1, vec![("test".into(), '1')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 1234
              │    ^^ test
        "},
        indoc::indoc! {"
            msmstttas
            ssmsssslls1111s
        "},
    );
}

#[test]
fn test_render_line_break_2() {
    // "1234\r\n5678\r\n90ab\r\ncdef\r\n"
    let lines = ["1234", "5678", "90ab", "cdef"];
    let mut builder = Snippet::builder(1);
    for line in &lines {
        for c in line.chars() {
            builder.push_char(c, 1, false);
        }
        builder.next_line(2); // two-unit line break (CRLF)
    }
    let snippet = builder.finish();

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(4..4, &ANNOT_STYLE_1, vec![("test".into(), '1')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 1234
              │     ^ test
        "},
        indoc::indoc! {"
            msmstttts
            ssmsssssls1111s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(4..5, &ANNOT_STYLE_1, vec![("test".into(), '1')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 1234
              │     ^ test
        "},
        indoc::indoc! {"
            msmstttts
            ssmsssssls1111s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(5..5, &ANNOT_STYLE_1, vec![("test".into(), '1')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 1234
              │     ^ test
        "},
        indoc::indoc! {"
            msmstttts
            ssmsssssls1111s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(5..6, &ANNOT_STYLE_1, vec![("test".into(), '1')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 1234
              │     ^ test
        "},
        indoc::indoc! {"
            msmstttts
            ssmsssssls1111s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(4..6, &ANNOT_STYLE_1, vec![("test".into(), '1')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 1234
              │     ^ test
        "},
        indoc::indoc! {"
            msmstttts
            ssmsssssls1111s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(3..4, &ANNOT_STYLE_1, vec![("test".into(), '1')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 1234
              │    ^ test
        "},
        indoc::indoc! {"
            msmstttas
            ssmssssls1111s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(3..5, &ANNOT_STYLE_1, vec![("test".into(), '1')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 1234
              │    ^^ test
        "},
        indoc::indoc! {"
            msmstttas
            ssmsssslls1111s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(3..6, &ANNOT_STYLE_1, vec![("test".into(), '1')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 1234
              │    ^^ test
        "},
        indoc::indoc! {"
            msmstttas
            ssmsssslls1111s
        "},
    );
}

#[test]
fn test_render_eof() {
    // "1234"
    let mut builder = Snippet::builder(1);
    for c in "1234".chars() {
        builder.push_char(c, 1, false);
    }
    let snippet = builder.finish();

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(4..4, &ANNOT_STYLE_1, vec![("test".into(), '1')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 1234
              │     ^ test
        "},
        indoc::indoc! {"
            msmstttts
            ssmsssssls1111s
        "},
    );

    // beyond EOF spans

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(4..5, &ANNOT_STYLE_1, vec![("test".into(), '1')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 1234
              │     ^ test
        "},
        indoc::indoc! {"
            msmstttts
            ssmsssssls1111s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(5..5, &ANNOT_STYLE_1, vec![("test".into(), '1')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 1234
              │     ^ test
        "},
        indoc::indoc! {"
            msmstttts
            ssmsssssls1111s
        "},
    );

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(5..6, &ANNOT_STYLE_1, vec![("test".into(), '1')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 1234
              │     ^ test
        "},
        indoc::indoc! {"
            msmstttts
            ssmsssssls1111s
        "},
    );
}

#[test]
fn test_render_line_numbers() {
    // "1234\n5678\n90ab\ncdef\n", starting at line number 99
    let lines = ["1234", "5678", "90ab", "cdef"];
    let mut builder = Snippet::builder(99);
    for line in &lines {
        for c in line.chars() {
            builder.push_char(c, 1, false);
        }
        builder.next_line(1);
    }
    let snippet = builder.finish();

    let mut annots = Annotations::new(&snippet, &MAIN_STYLE);
    annots.add_annotation(1..4, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(10..12, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);

    let mut output = TestOutput {
        text: String::new(),
        styles: String::new(),
    };
    let max_line_no_width = annots.max_line_no_width();
    annots.render(max_line_no_width, 0, 0, &mut output);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
             99 │ 1234
                │  ^^^ test 1
                ·
            101 │ 90ab
                │ -- test 2
        "},
        indoc::indoc! {"
            smmsmstaaas
            ssssmssllls111111s
            ssmmms
            mmmsmsbbtts
            ssssmsLLs222222s
        "},
    );
}

#[test]
fn test_render_different_discontinuity() {
    // "1234\n5678\n90ab\ncdef\n"
    let lines = ["1234", "5678", "90ab", "cdef"];
    let mut builder = Snippet::builder(1);
    for line in &lines {
        for c in line.chars() {
            builder.push_char(c, 1, false);
        }
        builder.next_line(1);
    }
    let snippet = builder.finish();

    let mut main_style = MAIN_STYLE;
    main_style.margin.as_mut().unwrap().discontinuity_chars = ['.', '.', '.'];

    let mut annots = Annotations::new(&snippet, &main_style);
    annots.add_annotation(1..4, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(10..12, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1 │ 1234
              │  ^^^ test 1
            ...
            3 │ 90ab
              │ -- test 2
        "},
        indoc::indoc! {"
            msmstaaas
            ssmssllls111111s
            mmms
            msmsbbtts
            ssmsLLs222222s
        "},
    );
}

#[test]
fn test_render_no_margin_single_line() {
    // "1234\n5678\n90ab\ncdef\n"
    let lines = ["1234", "5678", "90ab", "cdef"];
    let mut builder = Snippet::builder(1);
    for line in &lines {
        for c in line.chars() {
            builder.push_char(c, 1, false);
        }
        builder.next_line(1);
    }
    let snippet = builder.finish();

    let mut main_style = MAIN_STYLE;
    main_style.margin = None;

    let mut annots = Annotations::new(&snippet, &main_style);
    annots.add_annotation(1..4, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(10..12, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);

    let mut output = TestOutput {
        text: String::new(),
        styles: String::new(),
    };
    let max_line_no_width = annots.max_line_no_width();
    annots.render(max_line_no_width, 0, 0, &mut output);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
            1234
             ^^^ test 1

            90ab
            -- test 2
        "},
        indoc::indoc! {"
            taaas
            sllls111111s
            s
            bbtts
            LLs222222s
        "},
    );
}

#[test]
fn test_render_no_margin_multi_line() {
    // "1234\n5678\n90ab\ncdef\n"
    let lines = ["1234", "5678", "90ab", "cdef"];
    let mut builder = Snippet::builder(1);
    for line in &lines {
        for c in line.chars() {
            builder.push_char(c, 1, false);
        }
        builder.next_line(1);
    }
    let snippet = builder.finish();

    let mut main_style = MAIN_STYLE;
    main_style.margin = None;

    let mut annots = Annotations::new(&snippet, &main_style);
    annots.add_annotation(0..11, &ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(6..18, &ANNOT_STYLE_2, vec![("test 2".into(), '2')]);
    test_render(
        &annots,
        0,
        0,
        indoc::indoc! {"
             ╭ 1234
             │ 5678
            ╭│──-
            ││ 90ab
            │╰─^ test 1
            │  cdef
            ╰────- test 2
        "},
        indoc::indoc! {"
            slsaaaas
            slstbbbs
            LlLLLs
            Llsattts
            Lllls111111s
            Lssbbbts
            LLLLLLs222222s
        "},
    );
}
