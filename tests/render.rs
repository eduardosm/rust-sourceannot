#![warn(
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_qualifications
)]
#![forbid(unsafe_code)]

use sourceannot::{AnnotStyle, Annotations, MainStyle, MarginStyle, SourceSnippet};

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

fn gather_styles(rendered: &[(String, char)]) -> String {
    let mut r = String::new();
    for (text, style) in rendered.iter() {
        for chr in text.chars() {
            r.push(*style);
            if chr == '\n' {
                r.push('\n');
            }
        }
    }
    r
}

#[test]
fn test_render_single_line_1() {
    let source = "1234\n5678\n90ab\ncdef\n";
    let snippet = SourceSnippet::build_from_utf8(1, source.as_bytes(), 4);

    let mut annots = Annotations::new(&snippet, MAIN_STYLE);
    annots.add_annotation(1..4, ANNOT_STYLE_1, vec![("test".into(), '1')]);

    let rendered = annots.render(1, 0, 0);
    let text: String = rendered.iter().map(|(s, _)| s.as_str()).collect();
    let styles = gather_styles(&rendered);

    assert_eq!(
        text,
        indoc::indoc! {"
            1 │ 1234
              │  ^^^ test
        "},
    );
    assert_eq!(
        styles,
        indoc::indoc! {"
            msmstaaas
            ssmssllls1111s
        "},
    );
}

#[test]
fn test_render_single_line_2() {
    let source = "1234\n5678\n90ab\ncdef\n";
    let snippet = SourceSnippet::build_from_utf8(1, source.as_bytes(), 4);

    let mut annots = Annotations::new(&snippet, MAIN_STYLE);
    annots.add_annotation(1..4, ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(10..12, ANNOT_STYLE_2, vec![("test 2".into(), '2')]);

    let rendered = annots.render(1, 0, 0);
    let text: String = rendered.iter().map(|(s, _)| s.as_str()).collect();
    let styles = gather_styles(&rendered);

    assert_eq!(
        text,
        indoc::indoc! {"
            1 │ 1234
              │  ^^^ test 1
              · 
            3 │ 90ab
              │ -- test 2
        "},
    );
    assert_eq!(
        styles,
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
fn test_render_single_line_3() {
    let source = "1234\n5678\n90ab\ncdef\n";
    let snippet = SourceSnippet::build_from_utf8(1, source.as_bytes(), 4);

    let mut annots = Annotations::new(&snippet, MAIN_STYLE);
    annots.add_annotation(0..2, ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(3..4, ANNOT_STYLE_2, vec![("test 2".into(), '2')]);

    let rendered = annots.render(1, 0, 0);
    let text: String = rendered.iter().map(|(s, _)| s.as_str()).collect();
    let styles = gather_styles(&rendered);

    assert_eq!(
        text,
        indoc::indoc! {"
            1 │ 1234
              │ ^^ - test 2
              │ │
              │ test 1
        "},
    );
    assert_eq!(
        styles,
        indoc::indoc! {"
            msmsaatbs
            ssmsllsLs222222s
            ssmsls
            ssms111111s
        "},
    );
}

#[test]
fn test_render_single_line_4() {
    let source = "1234\n5678\n90ab\ncdef\n";
    let snippet = SourceSnippet::build_from_utf8(1, source.as_bytes(), 4);

    let mut annots = Annotations::new(&snippet, MAIN_STYLE);
    annots.add_annotation(1..2, ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(0..4, ANNOT_STYLE_2, vec![("test 2".into(), '2')]);

    let rendered = annots.render(1, 0, 0);
    let text: String = rendered.iter().map(|(s, _)| s.as_str()).collect();
    let styles = gather_styles(&rendered);

    assert_eq!(
        text,
        indoc::indoc! {"
            1 │ 1234
              │ -^--
              │ ││
              │ │test 1
              │ test 2
        "},
    );
    assert_eq!(
        styles,
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
fn test_render_multi_line_1() {
    let source = "1234\n5678\n90ab\ncdef\n";
    let snippet = SourceSnippet::build_from_utf8(1, source.as_bytes(), 4);

    let mut annots = Annotations::new(&snippet, MAIN_STYLE);
    annots.add_annotation(1..12, ANNOT_STYLE_1, vec![("test".into(), '1')]);

    let rendered = annots.render(1, 0, 0);
    let text: String = rendered.iter().map(|(s, _)| s.as_str()).collect();
    let styles = gather_styles(&rendered);

    assert_eq!(
        text,
        indoc::indoc! {"
            1 │   1234
              │ ╭──^
              · │ 
            3 │ │ 90ab
              │ ╰──^ test
        "},
    );
    assert_eq!(
        styles,
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
fn test_render_multi_line_2() {
    let source = "1234\n5678\n90ab\ncdef\n";
    let snippet = SourceSnippet::build_from_utf8(1, source.as_bytes(), 4);

    let mut annots = Annotations::new(&snippet, MAIN_STYLE);
    annots.add_annotation(0..11, ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(6..18, ANNOT_STYLE_2, vec![("test 2".into(), '2')]);

    let rendered = annots.render(1, 0, 0);
    let text: String = rendered.iter().map(|(s, _)| s.as_str()).collect();
    let styles = gather_styles(&rendered);

    assert_eq!(
        text,
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
        styles,
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
fn test_render_multi_line_3() {
    let source = "1234\n5678\n90ab\ncdef\n";
    let snippet = SourceSnippet::build_from_utf8(1, source.as_bytes(), 4);

    let mut annots = Annotations::new(&snippet, MAIN_STYLE);
    annots.add_annotation(0..7, ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(6..18, ANNOT_STYLE_2, vec![("test 2".into(), '2')]);

    let rendered = annots.render(1, 0, 0);
    let text: String = rendered.iter().map(|(s, _)| s.as_str()).collect();
    let styles = gather_styles(&rendered);

    assert_eq!(
        text,
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
        styles,
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
fn test_render_zero_len_span() {
    let source = "1234\n5678\n90ab\ncdef\n";
    let snippet = SourceSnippet::build_from_utf8(1, source.as_bytes(), 4);

    let mut annots = Annotations::new(&snippet, MAIN_STYLE);
    annots.add_annotation(1..1, ANNOT_STYLE_1, vec![("test".into(), '1')]);

    let rendered = annots.render(1, 0, 0);
    let text: String = rendered.iter().map(|(s, _)| s.as_str()).collect();
    let styles = gather_styles(&rendered);

    assert_eq!(
        text,
        indoc::indoc! {"
            1 │ 1234
              │  ^ test
        "},
    );
    assert_eq!(
        styles,
        indoc::indoc! {"
            msmstttts
            ssmssls1111s
        "},
    );
}

#[test]
fn test_render_tab() {
    let source = "1234\n\t5678\n";
    let snippet = SourceSnippet::build_from_utf8(1, source.as_bytes(), 4);

    let mut annots = Annotations::new(&snippet, MAIN_STYLE);
    annots.add_annotation(1..3, ANNOT_STYLE_1, vec![("test 1".into(), '1')]);
    annots.add_annotation(5..6, ANNOT_STYLE_2, vec![("test 2".into(), '2')]);

    let rendered = annots.render(1, 0, 0);
    let text: String = rendered.iter().map(|(s, _)| s.as_str()).collect();
    let styles = gather_styles(&rendered);

    assert_eq!(
        text,
        indoc::indoc! {"
            1 │ 1234
              │  ^^ test 1
            2 │     5678
              │ ---- test 2
        "},
    );
    assert_eq!(
        styles,
        indoc::indoc! {"
            msmstaats
            ssmsslls111111s
            msmsbbbbtttts
            ssmsLLLLs222222s
        "},
    );
}
