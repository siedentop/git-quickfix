# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.3] - 2020-10-03

### Removed

- Removed `--force` option to overwrite branches. It would be possible to
  provide this again in the future, please let me know if you want it.

### Changed

- Changes should already be committed. Thus it behaves now as something akin to
  `cherry-pick --onto`.
- Provide alias `git qf`.
- Try to read the default remote branch automatically. Unfortunately, still
  hard-coding main, master, devel (in that order).

### Fixed

- Pick up the git repo also from subdirectories.
- Reduce binary size.
- Reset starting branch.

## [0.0.2] - 2020-09-23

### Added

- If no message is provided, the users default editor will open.
- Logging is used instead of println.
- color_eyre is used for error handling

## [0.0.1] - 2020-09-22

### Added

- Initial Release
