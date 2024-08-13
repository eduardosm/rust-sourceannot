#![warn(
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_qualifications
)]
#![forbid(unsafe_code)]
#![no_std]

//! A library to render snippets of source code with annotations.
//! It is meant to be used as a building block for compiler diagnostics.
//!
//! This crate is `#![no_std]`, but it depends on `alloc`.
//!
//! # Example
//!
//! ```
//! // Some source code
//! let source = indoc::indoc! {r#"
//!     fn main() {
//!         println!("Hello, world!");
//!     }
//! "#};
//!
//! // Create the snippet
//! let snippet = sourceannot::SourceSnippet::build_from_utf8(1, source.as_bytes(), 4);
//!
//! // Styles are generic over the type of the metadata that accompanies each
//! // chunk of rendered text. In this example, we will use the following enum:
//! #[derive(Copy, Clone, Debug, PartialEq, Eq)]
//! enum Color {
//!     Default,
//!     Red,
//!     Green,
//!     Blue,
//! }
//! // If do not you need this per-chunk metadata, you can use `()` instead.
//!
//! // Define the styles
//! // Use Unicode box drawing characters
//! let main_style = sourceannot::MainStyle {
//!     margin: Some(sourceannot::MarginStyle {
//!         line_char: '│',
//!         dot_char: '·',
//!         meta: Color::Blue,
//!     }),
//!     horizontal_char: '─',
//!     vertical_char: '│',
//!     top_vertical_char: '╭',
//!     top_corner_char: '╭',
//!     bottom_corner_char: '╰',
//!     spaces_meta: Color::Default,
//!     text_normal_meta: Color::Default,
//!     text_alt_meta: Color::Default,
//! };
//!
//! // You can use a different style for each annotation, but in
//! // this example we will use the same style for all of them.
//! let annot_style = sourceannot::AnnotStyle {
//!     caret: '^',
//!     text_normal_meta: Color::Red,
//!     text_alt_meta: Color::Red,
//!     line_meta: Color::Red,
//! };
//!
//! // Create the annotations
//! let mut annotations = sourceannot::Annotations::new(&snippet, main_style);
//!
//! annotations.add_annotation(
//!     0..44,
//!     annot_style,
//!     vec![("this is the `main` function".into(), Color::Red)],
//! );
//! annotations.add_annotation(
//!     16..24,
//!     annot_style,
//!     vec![("this is a macro invocation".into(), Color::Red)],
//! );
//!
//! // Render the snippet with annotations
//! let max_line_no_width = annotations.max_line_no_width();
//! let rendered = annotations.render(max_line_no_width, 0, 0);
//!
//! // `rendered` is a `Vec<(String, Color)>`, which you could print with
//! // your favorite terminal coloring library. In this example, we will
//! // ignore the colors.
//!
//! for (chunk, _) in rendered.iter() {
//!     eprint!("{chunk}");
//! }
//!
//! # let rendered = rendered.iter().map(|(s, _)| s.as_str()).collect::<String>();
//! # assert_eq!(
//! #     rendered,
//! #     indoc::indoc! {"
//! #         1 │ ╭ fn main() {
//! #         2 │ │     println!(\"Hello, world!\");
//! #           │ │     ^^^^^^^^ this is a macro invocation
//! #         3 │ │ }
//! #           │ ╰─^ this is the `main` function
//! #     "},
//! # );
//! ```
//!
//! The output will look like this:
//!
//! ```text
//! 1 │ ╭ fn main() {
//! 2 │ │     println!(\"Hello, world!\");
//!   │ │     ^^^^^^^^ this is a macro invocation
//! 3 │ │ }
//!   │ ╰─^ this is the `main` function
//! ```

extern crate alloc;

mod annots;
mod range_set;
mod snippet;

pub use annots::Annotations;
pub use snippet::SourceSnippet;

/// The general style of an annotated snippet.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MainStyle<M> {
    /// The style of the margin.
    ///
    /// If `None`, there will not be any margin at all.
    pub margin: Option<MarginStyle<M>>,

    /// Character used to draw horizontal lines of multi-line annotations.
    pub horizontal_char: char,

    /// Character used to draw vertical lines of multi-line annotations.
    pub vertical_char: char,

    /// Character used to draw the top corner of multi-line annotations
    /// that start at the first column.
    pub top_vertical_char: char,

    /// Character used to draw the top corner of multi-line annotations.
    pub top_corner_char: char,

    /// Character used to draw the bottom corner of multi-line annotations.
    pub bottom_corner_char: char,

    /// Metadata that accompanies spaces.
    pub spaces_meta: M,

    /// Metadata that accompanies unannotated normal text.
    pub text_normal_meta: M,

    /// Metadata that accompanies unannotated alternative text.
    pub text_alt_meta: M,
}

/// The style of the margin of an annotated snippet.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MarginStyle<M> {
    /// Character used to draw the vertical separator of the margin.
    pub line_char: char,

    /// Character used to draw discontinuities in the vertical separator
    /// of the margin durin.
    pub dot_char: char,

    /// Metadata that accompanies margin characters.
    pub meta: M,
}

/// The style of a particular annotation.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AnnotStyle<M> {
    /// Caret character used to point to the annotated text.
    pub caret: char,

    /// Metadata that accompanies annotated normal text.
    pub text_normal_meta: M,

    /// Metadata that accompanies annotated alternative text.
    pub text_alt_meta: M,

    /// Metadata that accompanies annotation drawings.
    pub line_meta: M,
}
