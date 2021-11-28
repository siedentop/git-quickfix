// Require that
// 1) There shall be afterwards a branch. The commits should be going backwards: (1) the content is the same as the cherry-pick commit. (2) the next commit is the same as origin/main
// 2) The original quickfix commit should not be present on the original branch. (and opposite if --keep is provided)
// 3) Afterwards making a commit should be possible.
// 4) Afterwards, making another commit and repeating the exercise should also work. (It might not because of "cannot create a tree from a not fully merged index.; class=Index (10); code=Unmerged (-10)")

// Test from a subdirectory.

// Error if branch already exist? Unless --force is given.

// Test with untracked changes present.

// Test that the stash-list is not touched afterwards if the --stash option is used, and if it is not used.
// Test that stashing and a subsequent error leaves the repo untouched (i.e. the stash is popped).
