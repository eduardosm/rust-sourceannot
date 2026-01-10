# Changelog

## 0.3.0 (unreleased)

### Breaking

- `Annotations::render` now passes output to a new `Output` trait instead of
  returning a `Vec`.
- `SourceSnippet` has been renamed to `Snippet`.
- `SnippetBuilder` has been added to build custom `Snippet`s.
- `Snippet::get_line_col()` has been removed.
- `Snipper::build_from_utf8()`, `Snipper::build_from_utf8_ex()`,
  `Snippet::build_from_latin1()` and `Snippet::build_from_latin1_ex()` functions
  have been removed.
- New `Snipper::with_utf8()`, `Snipper::with_utf8_bytes()`,
  `Snippet::with_latin1()` and `Snippet::with_chars()` functions have been
  added.
- `dot_char` in `MarginStyle` has been renamed to `discontinuity_chars`, which now
  takes three characters.
- A new `std` feature, which depends on libstd has been enabled. Default features
  need to be disabled to support `no_std`.

### Fixed

- Fixed handling of spans that point to line breaks or zero-width fragments.
- Allow `on_control` and `on_invalid` (from `SourceSnippet::build_from_utf8_ex`
  and `SourceSnippet::build_from_latin1_ex`) to return strings with a UTF-8 length
  larger than 127 bytes or a width larger than 127.
- Line numbers in margins are now correctly aligned to the right.

### Changed

- `Annotations::render()` does not require `M: Clone` anymore.

### Other

- Minimum Supported Rust Version (MSRV) has been bumped to 1.85.

## 0.2.1 (2024-08-13)

### Added

- `no_std` support.

## 0.2.0 (2024-04-24)

### Added

- New functions to build snippets from Latin-1 (ISO 8859-1) sources.

### Changed

- CRLF sequences are now treated as a line break. Previously, they were treated
  as a control character followed by a line break.

### Fixed

- Documentation mistakes and typos.

## 0.1.1 (2024-03-30)

### Changed

- Improve `Debug` implementation of an internal type.

### Other

- Add repository URL to `Cargo.toml`.

## 0.1.0 (2024-03-23)

- Initial release
