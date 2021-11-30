# Git QuickFix

_Quickfix_ allows you to commit changes in your git repository to a new branch
without leaving the current branch.

## Motivation

I have often written a patch for some minor blemish that caught my attention. I
found it annoying to have to stage my main changes, switch to a new branch
created off the main branch, commit and push the patch and then switch back to
the original branch.

#### Benefits

- Minimize context switching.
- Much quicker on large repositories, where branch switching takes significant
  time.
- Everything happens in memory, so tools and IDEs watching the file system will
  not get confused.

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
   - With `--remove` the last commit will be removed from the original branch.
   - Add `--autostash` if you have local changes that you want to temporarily stash.

The cherry-pick is done in memory. This means your working directory will not be
modified.

The quickfix commit will be kept (changed in v0.0.5) unless `--remove` is provided.

## Installation

You can use Cargo to install quickfix.

Cargo is available for all major platforms. How to install Cargo is described here: https://doc.rust-lang.org/cargo/getting-started/installation.html

```shell
cargo install git-quickfix
```

## Known Issues

- Default branches from origin are picked up through a hard-coded list. Patches
  welcome. If the default branch on the remote is not called main, master,
  devel, or develop, you have to supply it manually. Similarly, if the remote is not
  called 'origin' you will also have to supply the branch name manually using the
  `--onto` option. Please [create an issue][ticket] if this bothers you.

- Won't fix: `--push` use the shell to push the changes. Benefits: All proxy,
  auth and other configs are picked up. But if does not feel right.

[ticket]: https://github.com/siedentop/git-quickfix/issues/new/choose

## FAQ

- _Can it handle multiple commits?_ -- No. I am trying to keep the tool simple.
  However, I originally started with this. If you see a need, please [create a
  feature request][ticket].
- _Are commit hooks_ considered? -- Not really. While the original commit will
  have pre- and post-commit hooks run, the cherry-pick itself does not run the
  pre-commit hook. As far as I understand, this may be the git [behavior][1], or
  maybe [not][2]? None of it matters, as libgit2 does [not support][3] hooks. If
  you have a use for a particular git-hook, please create a [new
  ticket][ticket].
- _How does this work?_ -- _libgit2_ provides a raw cherry-pick method. This
  works in-memory, meaning that the working directory (i.e. the checked out
  files) can be left untouched. This method returns a raw Index object, which I
  use to create a new commit.

[1]: http://git.661346.n2.nabble.com/cherry-pick-pre-commit-hook-td5815961.html
[2]:
  https://public-inbox.org/git/CAPig+cTT11J00aRO1gO06O6j5zdf4y6XRJhG5X7ZFeP6n7TOGQ@mail.gmail.com/T/
[3]: https://github.com/libgit2/libgit2/issues/964

## Related Work

* `git absorb`: https://github.com/tummychow/git-absorb
* `git trim`: https://github.com/foriequal0/git-trim
