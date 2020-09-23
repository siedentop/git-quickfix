# Git QuickFix

Quickfix allows you to commit changes in your git repository to a new branch
without leaving the current branch.

## Motivation

I have often written a patch for some minor blemish that caught my attention. I
found it annoying to have to stage my main changes, switch to a new branch
created off the main branch, commit and push the patch and then switch back to
the original branch.

## How it works

1. Stage the changes you would like to patch. `git add --patch` is a good way to
   do this.
2. `git quickfix --push <new_branch>`
3. `git quickfix --help` provides more options.

The new commit and the new branch are both created in memory. This means your
working directory will not be modified. Unless `--keep` is provided, the staged
changes will be removed.

## Installation

You can use Cargo to install quickfix.

```shell
cargo install git-quickfix
```

## Known Issues

- Calling from a subdirectory of the git repo is not implemented yet.
- Git configured editors (for the commit message) will not be picked up. Neither
  will a git commit-msg template. However, the EDITOR and VISUAL environment
  variables will be heeded.

### TODO

- Add Github action
- Fix unwraps
- Test gitdir
