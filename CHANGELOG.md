# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.3] - 2022-06-07

Updated some dependencies to work around build failures and security
vulnerabilities.

## [1.0.2] - 2020-03-06

### Fixed

* The `--generate-completions` option mistakenly still required a filename to
  an action file.

### Added

* All shells supported by the `--generate-completions` command are now
  mentioned in the README.

## [1.0.1] - 2020-03-06

### Added

* Some reference links in the documentation to Hydra and a YAML introduction.
  ([@ngirard](https://github.com/ngirard))

### Fixed

* Show correct version number in `man` pages.
* Added `--generate-completions` to `man` page.

### Changed

* Updated dependencies and to Rust 2018 edition.
  ([@ngirard](https://github.com/ngirard))

## [1.0.0] - 2018-01-19

Initial release.

[Unreleased]: https://github.com/Mange/tydra/compare/v1.0.3...HEAD
[1.0.2]: https://github.com/Mange/tydra/releases/tag/v1.0.3
[1.0.2]: https://github.com/Mange/tydra/releases/tag/v1.0.2
[1.0.1]: https://github.com/Mange/tydra/releases/tag/v1.0.1
[1.0.0]: https://github.com/Mange/tydra/releases/tag/v1.0.0
