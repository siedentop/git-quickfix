# Git QuickFix

Quickfix allows you to commit changes in your git repository to a new branch
without leaving the current branch.

## Motivation

I have often written a patch for some minor blemish that caught my attention. I
found it annoying to have to stage my main changes, switch to a new branch
created off the main branch, commit and push the patch and then switch back to
the original branch.

#### Benefits

- Minimize context switching.
- Much quicker on large repositories, where branch switching take significant
  time.
- Everything happens in memory, so tools watching the file system will not get
  confused.

## How it works

1. Commit the changes.
2. `git quickfix --push <new_branch>`

   - This will create a new branch named `<new_branch>` based-off `origin/main`
     (or whatever the remote default branch is).
   - The last commit on your current branch will be cherry-picked onto this new
     branch.
   - Leave out `--push` if you do not want to push the new branch to _origin_.

3. `git quickfix --help` provides more options.
   - With `--onto <branch>` you can modify the branch from which `<new_branch>`
     is based-off.
   - With `--keep` you can keep the last commit on the current branch.

The new commit and the new branch are both created in memory. This means your
working directory will not be modified. Unless `--keep` is provided, the staged
changes will be removed.

You can also use the alias **qf**: `git qf`.

## Installation

You can use Cargo to install quickfix.

```shell
cargo install git-quickfix
```

## Known Issues

- A dirty index or modified working directory will likely not work.
- Default branches from origin are picked up through a hard-coded list. Patches
  welcome.

### TODO

- Add Github action
- Fix unwraps
- `--push` will use the Shell to push the changes.
