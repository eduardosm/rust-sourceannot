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
//!
//! This crate is meant to be used as a building block for compiler diagnostics
//! (error reporting, warnings, lints, etc.).
//!
//! This crate is `#![no_std]`, but it depends on `alloc`.
//!
//! # Spans and positions
//!
//! Annotation spans are [`Range<usize>`](core::ops::Range) indices into the
//! snippet's *source unit* sequence (see [`Snippet`]). The exact unit depends
//! on how the snippet was built:
//!
//! - [`Snippet::with_utf8()`] uses **byte offsets** into the original `&str`.
//! - [`Snippet::with_utf8_bytes()`] uses **byte offsets** into the original `&[u8]`.
//! - [`Snippet::with_latin1()`] uses **byte offsets** into the original `&[u8]`.
//! - [`Snippet::with_chars()`] uses **[`char`] indices** into the original
//!   character sequence.
//!
//! These indices are *not* indices into the rendered output: some characters
//! will be replaced with some representation (for example, tabs are replaced
//! with spaces, some control characters are replaced, and invalid UTF-8 can
//! be represented as `�` or `<XX>`). The library keeps the mapping so that
//! spans still line up with what is shown.
//!
//! # Output flexibility
//!
//! Rendering is backend-agnostic: the library emits a stream of UTF-8 fragments
//! tagged with metadata, and an [`Output`] implementation decides what to do
//! with them.
//!
//! This lets you render to plain text (e.g. a [`String`](alloc::string::String)
//! or [`PlainOutput`]), or integrate with your own styling system (terminal colors,
//! HTML, etc.).
//!
//! # Cargo features
//!
//! - `std` (enabled by default): enables features that depend on [`std`],
//!   currently [`PlainOutput`] for writing rendered annotations to any
//!   [`std::io::Write`].
//!
//! When the `std` feature is disabled, this crate is `#![no_std]` but still
//! depends on [`alloc`].
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
//! let snippet = sourceannot::Snippet::with_utf8(
//!     1,
//!     source,
//!     4,
//!     sourceannot::ControlCharStyle::Hexadecimal,
//!     true,
//! );
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
//! // Render the snippet with annotations. `PlainOutput` can write to any
//! // `std::io::Write` ignoring colors. But you could use your favorite terminal
//! // coloring library with a wrapper that implements the `Output` trait.
//! let max_line_no_width = annotations.max_line_no_width();
//! annotations
//!     .render(
//!         max_line_no_width,
//!         0,
//!         0,
//!         sourceannot::PlainOutput(std::io::stderr().lock()),
//!     )
//!     .expect("failed to write to stderr");
//!
//! // You can also render to a string, which also ignores colors.
//! let mut rendered = String::new();
//! annotations.render(max_line_no_width, 0, 0, &mut rendered);
//!
//! # assert_eq!(
//! #     rendered,
//! #     indoc::indoc! {r#"
//! #         1 │ ╭ fn main() {
//! #         2 │ │     println!("Hello, world!");
//! #           │ │     ^^^^^^^^ this is a macro invocation
//! #         3 │ │ }
//! #           │ ╰─^ this is the `main` function
//! #     "#},
//! # );
//! ```
//!
//! The output will look like this:
//!
//! ```text
//! 1 │ ╭ fn main() {
//! 2 │ │     println!("Hello, world!");
//!   │ │     ^^^^^^^^ this is a macro invocation
//! 3 │ │ }
//!   │ ╰─^ this is the `main` function
//! ```
//!
//! With an invalid UTF-8 source:
//!
//! ```
//! // Some source code
//! let source = indoc::indoc! {b"
//!     fn main() {
//!         println!(\"Hello, \xFFworld!\");
//!     }
//! "};
//!
//! // Create the snippet
//! let snippet = sourceannot::Snippet::with_utf8_bytes(
//!     1,
//!     source,
//!     4,
//!     sourceannot::ControlCharStyle::Hexadecimal,
//!     true,
//!     sourceannot::InvalidUtf8SeqStyle::Hexadecimal,
//!     true,
//! );
//!
//! // Assume styles from the previous example...
//! # #[derive(Copy, Clone, Debug, PartialEq, Eq)]
//! # enum Color {
//! #     Default,
//! #     Red,
//! #     Green,
//! #     Blue,
//! # }
//! # let main_style = sourceannot::MainStyle {
//! #     margin: Some(sourceannot::MarginStyle {
//! #         line_char: '│',
//! #         dot_char: '·',
//! #         meta: Color::Blue,
//! #     }),
//! #     horizontal_char: '─',
//! #     vertical_char: '│',
//! #     top_vertical_char: '╭',
//! #     top_corner_char: '╭',
//! #     bottom_corner_char: '╰',
//! #     spaces_meta: Color::Default,
//! #     text_normal_meta: Color::Default,
//! #     text_alt_meta: Color::Default,
//! # };
//! # let annot_style = sourceannot::AnnotStyle {
//! #     caret: '^',
//! #     text_normal_meta: Color::Red,
//! #     text_alt_meta: Color::Red,
//! #     line_meta: Color::Red,
//! # };
//!
//! let mut annotations = sourceannot::Annotations::new(&snippet, main_style);
//! annotations.add_annotation(
//!     0..45,
//!     annot_style,
//!     vec![("this is the `main` function".into(), Color::Red)],
//! );
//!
//! // Add a span that points to the invalid UTF-8 byte.
//! annotations.add_annotation(
//!     33..34,
//!     annot_style,
//!     vec![("this an invalid UTF-8 sequence".into(), Color::Red)],
//! );
//!
//! let max_line_no_width = annotations.max_line_no_width();
//! annotations
//!     .render(
//!         max_line_no_width,
//!         0,
//!         0,
//!         sourceannot::PlainOutput(std::io::stderr().lock()),
//!     )
//!     .expect("failed to write to stderr");
//!
//! # let mut rendered = String::new();
//! # annotations.render(max_line_no_width, 0, 0, &mut rendered);
//! # assert_eq!(
//! #     rendered,
//! #     indoc::indoc! {r#"
//! #         1 │ ╭ fn main() {
//! #         2 │ │     println!("Hello, <FF>world!");
//! #           │ │                      ^^^^ this an invalid UTF-8 sequence
//! #         3 │ │ }
//! #           │ ╰─^ this is the `main` function
//! #     "#},
//! # );
//! ```
//!
//! The output will look like this:
//!
//! ```text
//! 1 │ ╭ fn main() {
//! 2 │ │     println!("Hello, <FF>world!");
//!   │ │                      ^^^^ this an invalid UTF-8 sequence
//! 3 │ │ }
//!   │ ╰─^ this is the `main` function
//! ```

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

mod annots;
mod range_set;
mod snippet;

pub use annots::Annotations;
pub use snippet::{ControlCharStyle, InvalidUtf8SeqStyle, Snippet, SnippetBuilder};

/// Trait that consumes a rendered annotated snippet.
///
/// Rendering produces a stream of text fragments , each tagged with some
/// metadata `M` that describes how that fragment should be presented (for
/// example, a color/style).
///
/// You can implement this trait to plug in your preferred output backend:
/// plain text, terminal coloring, HTML, etc.
///
/// `M` is an implementor-defined metadata type. You can use `()` if you do not
/// need it.
///
/// # Example
///
/// A simple `Output` implementation that captures rendered fragments alongside
/// their metadata:
///
/// ```
/// #[derive(Copy, Clone, Debug, PartialEq, Eq)]
/// enum Style {
///     Normal,
///     Emph,
/// }
///
/// struct Capture(pub Vec<(String, Style)>);
///
/// impl sourceannot::Output<Style> for Capture {
///     type Error = std::convert::Infallible;
///
///     fn put_str(&mut self, text: &str, meta: &Style) -> Result<(), Self::Error> {
///         self.0.push((text.to_string(), *meta));
///         Ok(())
///     }
/// }
/// ```
pub trait Output<M> {
    /// Error type produced by this output backend.
    ///
    /// For example, it can be [`std::io::Error`] when writing to an I/O
    /// stream, or [`std::convert::Infallible`] when the output cannot fail.
    type Error;

    /// Writes a UTF-8 text fragment with associated metadata.
    fn put_str(&mut self, text: &str, meta: &M) -> Result<(), Self::Error>;

    /// Writes a single character with associated metadata.
    fn put_char(&mut self, ch: char, meta: &M) -> Result<(), Self::Error> {
        self.put_str(ch.encode_utf8(&mut [0; 4]), meta)
    }

    /// Writes formatted text with associated metadata.
    fn put_fmt(&mut self, args: core::fmt::Arguments<'_>, meta: &M) -> Result<(), Self::Error> {
        struct Adapter<'a, M, O: ?Sized + Output<M>> {
            output: &'a mut O,
            meta: &'a M,
            error: Option<O::Error>,
        }

        impl<'a, M, O: ?Sized + Output<M>> core::fmt::Write for Adapter<'a, M, O> {
            fn write_str(&mut self, s: &str) -> core::fmt::Result {
                self.output.put_str(s, self.meta).map_err(|e| {
                    self.error = Some(e);
                    core::fmt::Error
                })
            }
        }

        let mut writer = Adapter {
            output: self,
            meta,
            error: None,
        };
        core::fmt::write(&mut writer, args)
            .map_err(|_| {
                writer
                    .error
                    .unwrap_or_else(|| {
                        panic!("a formatting trait implementation returned an error when the underlying stream did not")
                    })
            })
    }
}

/// Writing to a [`String`](alloc::string::String) ignores metadata.
impl<M> Output<M> for &mut alloc::string::String {
    type Error = core::convert::Infallible;

    fn put_str(&mut self, text: &str, _meta: &M) -> Result<(), Self::Error> {
        self.push_str(text);
        Ok(())
    }

    fn put_char(&mut self, ch: char, _meta: &M) -> Result<(), Self::Error> {
        self.push(ch);
        Ok(())
    }

    fn put_fmt(&mut self, args: core::fmt::Arguments<'_>, _meta: &M) -> Result<(), Self::Error> {
        core::fmt::write(self, args).unwrap();
        Ok(())
    }
}

/// An [`Output`] implementor that writes to any [`std::io::Write`] ignoring
/// metadata.
#[cfg(feature = "std")]
pub struct PlainOutput<W: std::io::Write>(pub W);

#[cfg(feature = "std")]
impl<W: std::io::Write, M> Output<M> for PlainOutput<W> {
    type Error = std::io::Error;

    fn put_str(&mut self, text: &str, _meta: &M) -> Result<(), Self::Error> {
        self.0.write_all(text.as_bytes())
    }

    fn put_char(&mut self, ch: char, _meta: &M) -> Result<(), Self::Error> {
        let mut buf = [0; 4];
        let s = ch.encode_utf8(&mut buf);
        self.0.write_all(s.as_bytes())
    }

    fn put_fmt(&mut self, args: core::fmt::Arguments<'_>, _meta: &M) -> Result<(), Self::Error> {
        self.0.write_fmt(args)
    }
}

/// The general style of an annotated snippet.
///
/// This controls how the snippet and its annotations are drawn (margin,
/// connector lines, corners) and which metadata is attached to each text
/// fragment.
///
/// `M` is an output-backend-defined metadata type (often a "color/style"). It
/// is passed through to [`Output`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MainStyle<M> {
    /// The style of the margin.
    ///
    /// If `None`, there will not be any margin at all.
    pub margin: Option<MarginStyle<M>>,

    /// Character used to draw the horizontal connectors of multi-line annotations.
    pub horizontal_char: char,

    /// Character used to draw the vertical connector of multi-line annotations.
    pub vertical_char: char,

    /// Character used to draw the top corner of multi-line annotations that
    /// start at the first column.
    pub top_vertical_char: char,

    /// Character used to draw the top corner of multi-line annotations.
    pub top_corner_char: char,

    /// Character used to draw the bottom corner of multi-line annotations.
    pub bottom_corner_char: char,

    /// Metadata that accompanies spaces.
    ///
    /// This is used for padding and separator spaces inserted by the renderer.
    pub spaces_meta: M,

    /// Metadata that accompanies unannotated text.
    pub text_normal_meta: M,

    /// Metadata that accompanies unannotated alternate text.
    ///
    /// "Alternate text" refers to replacement text emitted when the renderer
    /// makes normally-invisible or potentially-confusing source elements
    /// explicit (for example, certain control characters or invalid UTF-8
    /// sequences, depending on snippet settings).
    pub text_alt_meta: M,
}

/// The style of the margin of an annotated snippet.
///
/// The margin is the left-hand area that typically contains line numbers and a
/// vertical separator.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MarginStyle<M> {
    /// Character used to draw the vertical separator of the margin.
    pub line_char: char,

    /// Character used to draw discontinuities in the vertical separator of the
    /// margin when intermediate source lines are omitted.
    pub dot_char: char,

    /// Metadata that accompanies margin characters.
    ///
    /// This applies to line numbers as well as the margin separator glyphs.
    pub meta: M,
}

/// The style of a particular annotation.
///
/// This controls the glyphs and metadata used to render a specific annotation
/// span (carets, connector lines, and label text).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AnnotStyle<M> {
    /// Caret character used to point to the annotated text.
    pub caret: char,

    /// Metadata that accompanies annotated text.
    pub text_normal_meta: M,

    /// Metadata that accompanies annotated alternate text.
    pub text_alt_meta: M,

    /// Metadata that accompanies annotation drawings.
    ///
    /// This applies to carets and connector lines.
    pub line_meta: M,
}
