#![warn(
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_qualifications
)]
#![forbid(unsafe_code)]

use sourceannot::{
    AnnotStyle, Annotations, ControlCharStyle, InvalidUtf8SeqStyle, MainStyle, MarginStyle,
    Utf8SourceSnippet,
};

const MAIN_STYLE: MainStyle<char> = MainStyle {
    margin: Some(MarginStyle {
        line_char: '│',
        dot_char: '·',
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

fn build_snippet(source: &(impl ?Sized + AsRef<[u8]>)) -> Utf8SourceSnippet<'_> {
    Utf8SourceSnippet::new(
        source.as_ref(),
        false,
        4,
        ControlCharStyle::Hexadecimal,
        true,
        InvalidUtf8SeqStyle::Replacement,
        true,
    )
}

#[test]
fn test_single_line_1() {
    let source = "1234\n5678\n90ab\ncdef\n";
    let snippet = build_snippet(source);

    let mut annots = Annotations::new(&snippet, 1, MAIN_STYLE);
    annots.add_annotation(1..4, ANNOT_STYLE_1, vec![("test".into(), '1')]);

    let mut output = TestOutput {
        text: String::new(),
        styles: String::new(),
    };
    let max_line_no_width = annots.max_line_no_width();
    annots.render(max_line_no_width, 0, 0, &mut output);

    assert_eq!(
        output.text,
        indoc::indoc! {"
            1 │ 1234
              │  ^^^ test
        "},
    );
    assert_eq!(
        output.styles,
        indoc::indoc! {"
            msmstaaas
            ssmssllls1111s
        "},
    );
}

#[test]
fn test_single_line_2() {
    let source = "1234\n5678\n90ab\ncdef\n";
    let snippet = build_snippet(source);

    let mut annots = Annotations::new(&snippet, 1, MAIN_STYLE);
    annots.add_annotation(1..4, ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(10..12, ANNOT_STYLE_2, vec![("test 2".into(), '2')]);

    let mut output = TestOutput {
        text: String::new(),
        styles: String::new(),
    };
    let max_line_no_width = annots.max_line_no_width();
    annots.render(max_line_no_width, 0, 0, &mut output);

    assert_eq!(
        output.text,
        indoc::indoc! {"
            1 │ 1234
              │  ^^^ test 1
              · 
            3 │ 90ab
              │ -- test 2
        "},
    );
    assert_eq!(
        output.styles,
        indoc::indoc! {"
            msmstaaas
            ssmssllls111111s
            ssmss
            msmsbbtts
            ssmsLLs222222s
        "},
    );
}

#[test]
fn test_single_line_3() {
    let source = "1234\n5678\n90ab\ncdef\n";
    let snippet = build_snippet(source);

    let mut annots = Annotations::new(&snippet, 1, MAIN_STYLE);
    annots.add_annotation(0..2, ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(3..4, ANNOT_STYLE_2, vec![("test 2".into(), '2')]);

    let mut output = TestOutput {
        text: String::new(),
        styles: String::new(),
    };
    let max_line_no_width = annots.max_line_no_width();
    annots.render(max_line_no_width, 0, 0, &mut output);

    assert_eq!(
        output.text,
        indoc::indoc! {"
            1 │ 1234
              │ ^^ - test 2
              │ │
              │ test 1
        "},
    );
    assert_eq!(
        output.styles,
        indoc::indoc! {"
            msmsaatbs
            ssmsllsLs222222s
            ssmsls
            ssms111111s
        "},
    );
}

#[test]
fn test_single_line_4() {
    let source = "1234\n5678\n90ab\ncdef\n";
    let snippet = build_snippet(source);

    let mut annots = Annotations::new(&snippet, 1, MAIN_STYLE);
    annots.add_annotation(1..2, ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(0..4, ANNOT_STYLE_2, vec![("test 2".into(), '2')]);

    let mut output = TestOutput {
        text: String::new(),
        styles: String::new(),
    };
    let max_line_no_width = annots.max_line_no_width();
    annots.render(max_line_no_width, 0, 0, &mut output);

    assert_eq!(
        output.text,
        indoc::indoc! {"
            1 │ 1234
              │ -^--
              │ ││
              │ │test 1
              │ test 2
        "},
    );
    assert_eq!(
        output.styles,
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
fn test_multi_line_1() {
    let source = "1234\n5678\n90ab\ncdef\n";
    let snippet = build_snippet(source);

    let mut annots = Annotations::new(&snippet, 1, MAIN_STYLE);
    annots.add_annotation(1..12, ANNOT_STYLE_1, vec![("test".into(), '1')]);

    let mut output = TestOutput {
        text: String::new(),
        styles: String::new(),
    };
    let max_line_no_width = annots.max_line_no_width();
    annots.render(max_line_no_width, 0, 0, &mut output);

    assert_eq!(
        output.text,
        indoc::indoc! {"
            1 │   1234
              │ ╭──^
              · │ 
            3 │ │ 90ab
              │ ╰──^ test
        "},
    );
    assert_eq!(
        output.styles,
        indoc::indoc! {"
            msmssstaaas
            ssmslllls
            ssmslss
            msmslsaatts
            ssmslllls1111s
        "},
    );
}

#[test]
fn test_multi_line_2() {
    let source = "1234\n5678\n90ab\ncdef\n";
    let snippet = build_snippet(source);

    let mut annots = Annotations::new(&snippet, 1, MAIN_STYLE);
    annots.add_annotation(0..11, ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(6..18, ANNOT_STYLE_2, vec![("test 2".into(), '2')]);

    let mut output = TestOutput {
        text: String::new(),
        styles: String::new(),
    };
    let max_line_no_width = annots.max_line_no_width();
    annots.render(max_line_no_width, 0, 0, &mut output);

    assert_eq!(
        output.text,
        indoc::indoc! {"
            1 │  ╭ 1234
            2 │  │ 5678
              │ ╭│──-
            3 │ ││ 90ab
              │ │╰─^ test 1
            4 │ │  cdef
              │ ╰────- test 2
        "},
    );
    assert_eq!(
        output.styles,
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
}

#[test]
fn test_multi_line_3() {
    let source = "1234\n5678\n90ab\ncdef\n";
    let snippet = build_snippet(source);

    let mut annots = Annotations::new(&snippet, 1, MAIN_STYLE);
    annots.add_annotation(0..18, ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(6..11, ANNOT_STYLE_2, vec![("test 2".into(), '2')]);

    let mut output = TestOutput {
        text: String::new(),
        styles: String::new(),
    };
    let max_line_no_width = annots.max_line_no_width();
    annots.render(max_line_no_width, 0, 0, &mut output);

    assert_eq!(
        output.text,
        indoc::indoc! {"
            1 │  ╭ 1234
            2 │  │ 5678
              │ ╭│──-
            3 │ ││ 90ab
              │ ╰│─- test 2
            4 │  │ cdef
              │  ╰───^ test 1
        "},
    );
    assert_eq!(
        output.styles,
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
}

#[test]
fn test_multi_line_4() {
    let source = "1234\n5678\n90ab\ncdef\n";
    let snippet = build_snippet(source);

    let mut annots = Annotations::new(&snippet, 1, MAIN_STYLE);
    annots.add_annotation(6..11, ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(0..18, ANNOT_STYLE_2, vec![("test 2".into(), '2')]);

    let mut output = TestOutput {
        text: String::new(),
        styles: String::new(),
    };
    let max_line_no_width = annots.max_line_no_width();
    annots.render(max_line_no_width, 0, 0, &mut output);

    assert_eq!(
        output.text,
        indoc::indoc! {"
            1 │ ╭  1234
            2 │ │  5678
              │ │╭──^
            3 │ ││ 90ab
              │ │╰─^ test 1
            4 │ │  cdef
              │ ╰────- test 2
        "},
    );
    assert_eq!(
        output.styles,
        indoc::indoc! {"
            msmsLssbbbbs
            msmsLsstaaas
            ssmsLlllls
            msmsLlsattts
            ssmsLllls111111s
            msmsLssbbbts
            ssmsLLLLLLs222222s
        "},
    );
}

#[test]
fn test_multi_line_5() {
    let source = "1234\n5678\n90ab\ncdef\n";
    let snippet = build_snippet(source);

    let mut annots = Annotations::new(&snippet, 1, MAIN_STYLE);
    annots.add_annotation(0..7, ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(6..18, ANNOT_STYLE_2, vec![("test 2".into(), '2')]);

    let mut output = TestOutput {
        text: String::new(),
        styles: String::new(),
    };
    let max_line_no_width = annots.max_line_no_width();
    annots.render(max_line_no_width, 0, 0, &mut output);

    assert_eq!(
        output.text,
        indoc::indoc! {"
            1 │ ╭ 1234
            2 │ │ 5678
              │ ╰──^ test 1
              │ ╭──-
              · │ 
            4 │ │ cdef
              │ ╰───- test 2
        "},
    );
    assert_eq!(
        output.styles,
        indoc::indoc! {"
            msmslsaaaas
            msmslsaabbs
            ssmslllls111111s
            ssmsLLLLs
            ssmsLss
            msmsLsbbbts
            ssmsLLLLLs222222s
        "},
    );
}

#[test]
fn test_multi_line_crlf() {
    let source = "1234\r\n5678\r\n90ab\r\ncdef\r\n";
    let snippet = build_snippet(source);

    let mut annots = Annotations::new(&snippet, 1, MAIN_STYLE);
    annots.add_annotation(1..14, ANNOT_STYLE_1, vec![("test".into(), '1')]);

    let mut output = TestOutput {
        text: String::new(),
        styles: String::new(),
    };
    let max_line_no_width = annots.max_line_no_width();
    annots.render(max_line_no_width, 0, 0, &mut output);

    assert_eq!(
        output.text,
        indoc::indoc! {"
            1 │   1234
              │ ╭──^
              · │ 
            3 │ │ 90ab
              │ ╰──^ test
        "},
    );
    assert_eq!(
        output.styles,
        indoc::indoc! {"
            msmssstaaas
            ssmslllls
            ssmslss
            msmslsaatts
            ssmslllls1111s
        "},
    );
}

#[test]
fn test_single_line_within_multi_line() {
    let source = "1234\n5678\n90ab\ncdef\n";
    let snippet = build_snippet(source);

    let mut annots = Annotations::new(&snippet, 1, MAIN_STYLE);
    annots.add_annotation(1..12, ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(6..9, ANNOT_STYLE_2, vec![("test 2".into(), '2')]);

    let mut output = TestOutput {
        text: String::new(),
        styles: String::new(),
    };
    let max_line_no_width = annots.max_line_no_width();
    annots.render(max_line_no_width, 0, 0, &mut output);

    assert_eq!(
        output.text,
        indoc::indoc! {"
            1 │   1234
              │ ╭──^
            2 │ │ 5678
              │ │  --- test 2
            3 │ │ 90ab
              │ ╰──^ test 1
        "},
    );
    assert_eq!(
        output.styles,
        indoc::indoc! {"
            msmssstaaas
            ssmslllls
            msmslstbbbs
            ssmslssLLLs222222s
            msmslsaatts
            ssmslllls111111s
        "},
    );
}

#[test]
fn test_zero_len_span() {
    let source = "1234\n5678\n90ab\ncdef\n";
    let snippet = build_snippet(source);

    let mut annots = Annotations::new(&snippet, 1, MAIN_STYLE);
    annots.add_annotation(1..1, ANNOT_STYLE_1, vec![("test".into(), '1')]);

    let mut output = TestOutput {
        text: String::new(),
        styles: String::new(),
    };
    let max_line_no_width = annots.max_line_no_width();
    annots.render(max_line_no_width, 0, 0, &mut output);

    assert_eq!(
        output.text,
        indoc::indoc! {"
            1 │ 1234
              │  ^ test
        "},
    );
    assert_eq!(
        output.styles,
        indoc::indoc! {"
            msmstttts
            ssmssls1111s
        "},
    );
}

#[test]
fn test_tab() {
    let source = "1234\n\t5678\n";
    let snippet = build_snippet(source);

    let mut annots = Annotations::new(&snippet, 1, MAIN_STYLE);
    annots.add_annotation(1..3, ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(5..6, ANNOT_STYLE_2, vec![("test 2".into(), '2')]);

    let mut output = TestOutput {
        text: String::new(),
        styles: String::new(),
    };
    let max_line_no_width = annots.max_line_no_width();
    annots.render(max_line_no_width, 0, 0, &mut output);

    assert_eq!(
        output.text,
        indoc::indoc! {"
            1 │ 1234
              │  ^^ test 1
            2 │     5678
              │ ---- test 2
        "},
    );
    assert_eq!(
        output.styles,
        indoc::indoc! {"
            msmstaats
            ssmsslls111111s
            msmsbbbbtttts
            ssmsLLLLs222222s
        "},
    );
}

#[test]
fn test_line_break_lf_1() {
    let source = "1234\n5678\n90ab\ncdef\n";
    let snippet = build_snippet(source);

    let mut annots = Annotations::new(&snippet, 1, MAIN_STYLE);
    annots.add_annotation(4..5, ANNOT_STYLE_1, vec![("test".into(), '1')]);

    let mut output = TestOutput {
        text: String::new(),
        styles: String::new(),
    };
    let max_line_no_width = annots.max_line_no_width();
    annots.render(max_line_no_width, 0, 0, &mut output);

    assert_eq!(
        output.text,
        indoc::indoc! {"
            1 │ 1234
              │     ^ test
        "},
    );
    assert_eq!(
        output.styles,
        indoc::indoc! {"
            msmstttts
            ssmsssssls1111s
        "},
    );
}

#[test]
fn test_line_break_lf_2() {
    let source = "1234\n5678\n90ab\ncdef\n";
    let snippet = build_snippet(source);

    let mut annots = Annotations::new(&snippet, 1, MAIN_STYLE);
    annots.add_annotation(3..5, ANNOT_STYLE_1, vec![("test".into(), '1')]);

    let mut output = TestOutput {
        text: String::new(),
        styles: String::new(),
    };
    let max_line_no_width = annots.max_line_no_width();
    annots.render(max_line_no_width, 0, 0, &mut output);

    assert_eq!(
        output.text,
        indoc::indoc! {"
            1 │ 1234
              │    ^^ test
        "},
    );
    assert_eq!(
        output.styles,
        indoc::indoc! {"
            msmstttas
            ssmsssslls1111s
        "},
    );
}

#[test]
fn test_line_break_crlf_1() {
    let source = "1234\r\n5678\r\n90ab\r\ncdef\r\n";
    let snippet = build_snippet(source);

    let mut annots = Annotations::new(&snippet, 1, MAIN_STYLE);
    annots.add_annotation(4..5, ANNOT_STYLE_1, vec![("test".into(), '1')]);

    let mut output = TestOutput {
        text: String::new(),
        styles: String::new(),
    };
    let max_line_no_width = annots.max_line_no_width();
    annots.render(max_line_no_width, 0, 0, &mut output);

    assert_eq!(
        output.text,
        indoc::indoc! {"
            1 │ 1234
              │     ^ test
        "},
    );
    assert_eq!(
        output.styles,
        indoc::indoc! {"
            msmstttts
            ssmsssssls1111s
        "},
    );
}

#[test]
fn test_line_break_crlf_2() {
    let source = "1234\r\n5678\r\n90ab\r\ncdef\r\n";
    let snippet = build_snippet(source);

    let mut annots = Annotations::new(&snippet, 1, MAIN_STYLE);
    annots.add_annotation(5..6, ANNOT_STYLE_1, vec![("test".into(), '1')]);

    let mut output = TestOutput {
        text: String::new(),
        styles: String::new(),
    };
    let max_line_no_width = annots.max_line_no_width();
    annots.render(max_line_no_width, 0, 0, &mut output);

    assert_eq!(
        output.text,
        indoc::indoc! {"
            1 │ 1234
              │     ^ test
        "},
    );
    assert_eq!(
        output.styles,
        indoc::indoc! {"
            msmstttts
            ssmsssssls1111s
        "},
    );
}

#[test]
fn test_line_break_crlf_3() {
    let source = "1234\r\n5678\r\n90ab\r\ncdef\r\n";
    let snippet = build_snippet(source);

    let mut annots = Annotations::new(&snippet, 1, MAIN_STYLE);
    annots.add_annotation(4..6, ANNOT_STYLE_1, vec![("test".into(), '1')]);

    let mut output = TestOutput {
        text: String::new(),
        styles: String::new(),
    };
    let max_line_no_width = annots.max_line_no_width();
    annots.render(max_line_no_width, 0, 0, &mut output);

    assert_eq!(
        output.text,
        indoc::indoc! {"
            1 │ 1234
              │     ^ test
        "},
    );
    assert_eq!(
        output.styles,
        indoc::indoc! {"
            msmstttts
            ssmsssssls1111s
        "},
    );
}

#[test]
fn test_line_break_crlf_4() {
    let source = "1234\r\n5678\r\n90ab\r\ncdef\r\n";
    let snippet = build_snippet(source);

    let mut annots = Annotations::new(&snippet, 1, MAIN_STYLE);
    annots.add_annotation(3..5, ANNOT_STYLE_1, vec![("test".into(), '1')]);

    let mut output = TestOutput {
        text: String::new(),
        styles: String::new(),
    };
    let max_line_no_width = annots.max_line_no_width();
    annots.render(max_line_no_width, 0, 0, &mut output);

    assert_eq!(
        output.text,
        indoc::indoc! {"
            1 │ 1234
              │    ^^ test
        "},
    );
    assert_eq!(
        output.styles,
        indoc::indoc! {"
            msmstttas
            ssmsssslls1111s
        "},
    );
}

#[test]
fn test_line_break_crlf_5() {
    let source = "1234\r\n5678\r\n90ab\r\ncdef\r\n";
    let snippet = build_snippet(source);

    let mut annots = Annotations::new(&snippet, 1, MAIN_STYLE);
    annots.add_annotation(3..6, ANNOT_STYLE_1, vec![("test".into(), '1')]);

    let mut output = TestOutput {
        text: String::new(),
        styles: String::new(),
    };
    let max_line_no_width = annots.max_line_no_width();
    annots.render(max_line_no_width, 0, 0, &mut output);

    assert_eq!(
        output.text,
        indoc::indoc! {"
            1 │ 1234
              │    ^^ test
        "},
    );
    assert_eq!(
        output.styles,
        indoc::indoc! {"
            msmstttas
            ssmsssslls1111s
        "},
    );
}

#[test]
fn test_line_numbers() {
    let source = "1234\n5678\n90ab\ncdef\n";
    let snippet = build_snippet(source);

    let mut annots = Annotations::new(&snippet, 99, MAIN_STYLE);
    annots.add_annotation(1..4, ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(10..12, ANNOT_STYLE_2, vec![("test 2".into(), '2')]);

    let mut output = TestOutput {
        text: String::new(),
        styles: String::new(),
    };
    let max_line_no_width = annots.max_line_no_width();
    annots.render(max_line_no_width, 0, 0, &mut output);

    assert_eq!(
        output.text,
        indoc::indoc! {"
             99 │ 1234
                │  ^^^ test 1
                · 
            101 │ 90ab
                │ -- test 2
        "},
    );
    assert_eq!(
        output.styles,
        indoc::indoc! {"
            smmsmstaaas
            ssssmssllls111111s
            ssssmss
            mmmsmsbbtts
            ssssmsLLs222222s
        "},
    );
}
