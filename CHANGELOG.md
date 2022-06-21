# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- Release Process: https://dev.to/sharkdp/my-release-checklist-for-rust-programs-1m33 -->

## [Upcoming]

## [0.1.0] - 2022-06-20

This version signifies that the project is as complete as possible. I do not expect
major updates (famous last words). Thank you to @rofrol, @betwo, @sharkdp, @msfjarvis for
their contributions to the project.

### Added

- Made `develop` (via `origin/develop`) a default target branch, as suggested by @msfjarvis [here](https://github.com/siedentop/git-quickfix/issues/7#issuecomment-791143253).
- Use `git branch -rl` to identify where '\*/HEAD' points to. Thanks to @rofrol in #11 for the suggestion. In case this fails, the previous solution of using one of _main_, _master_, _develop_, or _devel_ is used.

## [0.0.5] - 2021-11-29

Thanks to @betwo and @sharkdp for their suggestions on improvements.

### Changed

- The option `--stash` has been renamed to `--autostash`. [Issue #9](https://github.com/siedentop/git-quickfix/issues/9)
- The option `--keep` is now the default, and the opposite is now called `--remove`.
  Providing `--remove` will now drop the commit from the original branch. By default, the commit
  stays on the branch. [Issue #10](https://github.com/siedentop/git-quickfix/issues/8)

### Fixed

- Minor: Fix old 'suggestion' for '--stash' option.

### Removed

- Removed the `qf` alias. All votes were in favor [1].

[1]: https://github.com/siedentop/git-quickfix/issues/6

## [0.0.4] - 2020-11-10

### Fixed

- Prevent loss of data in case of uncommitted changes. I profoundly apologize to
  the affected user. [PR#2](https://github.com/siedentop/git-quickfix/pull/2).
  Thanks to Sebastian Buck (@betwo) for the changes.

### Added

- Option `--force` overwrites an existing branch. (Use `git reflog` in case of
  inadvertent use.)
- Option `--stash` auto stashes unstaged changes. Thanks to Sebastian Buck
  (@betwo) for the idea and implementation.

### Deprecated

- I am considering removing the `qf` alias in a later release. Please vote
  [here][1].

[1]: https://github.com/siedentop/git-quickfix/issues/6

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
