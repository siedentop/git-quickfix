use eyre::{eyre, Result};
use std::process;

use git2::{self, Repository, RepositoryState, ResetType};
use process::Command;

use color_eyre::{eyre::Report, Section};

extern crate log;

/// Wraps the three steps below into one, such that any error can be caught and
/// the git stash can be popped before exiting.
/// In many ways this is a poor-person's try-finally (or context manager in Python).
/// Using Drop led to multiple borrowing errors. Any improvements along those lines
/// are more than welcome.
pub fn wrapper_pick_and_clean(
    repo: &Repository,
    target_branch: &str,
    onto_branch: &str,
    force_new_branch: bool,
) -> Result<()> {
    assure_workspace_is_clean(repo)
        .suggestion("Consider auto-stashing your changes with --autostash.")
        .suggestion("Running this again with RUST_LOG=debug provides more details.")?;
    cherrypick_commit_onto_new_branch(repo, target_branch, onto_branch, force_new_branch)?;
    remove_commit_from_head(repo)?;
    Ok(())
}

/// The main functional function.
///
/// Please read the README for some further background.
pub fn cherrypick_commit_onto_new_branch(
    repo: &Repository,
    target_branch: &str,
    onto_branch: &str,
    force_new_branch: bool,
) -> Result<(), Report> {
    let main_commit = repo
        .revparse(onto_branch)?
        .from()
        .unwrap()
        .peel_to_commit()?;

    // Create the new branch
    let new_branch = repo
        .branch(target_branch, &main_commit, force_new_branch)
        .suggestion("Consider using --force to overwrite the existing branch")?;

    // Cherry-pick the HEAD onto the main branch but in memory.
    // Then create a new branch with that cherry-picked commit.
    let fix_commit = repo.head()?.peel_to_commit()?;
    if fix_commit.parent_count() != 1 {
        return Err(eyre!("Only works with non-merge commits"))
            .suggestion("Quickfixing a merge commit is not supported. If you meant to do this please file a ticket with your use case.");
    };

    // Cherry-pick (in memory)
    let mut index = repo.cherrypick_commit(&fix_commit, &main_commit, 0, None)?;
    let tree_oid = index.write_tree_to(repo)?;
    let tree = repo.find_tree(tree_oid)?;

    // The author is copied from the original commit. But the committer is set to the current user and timestamp.
    let signature = repo.signature()?;
    let message = fix_commit
        .message_raw()
        .ok_or_else(|| eyre!("Could not read the commit message."))
        .suggestion("Make sure the commit message contains only UTF-8 characters or try to manually cherry-pick the commit.")?;

    let commit_oid = repo
        .commit(
            new_branch.get().name(),
            &fix_commit.author(),
            &signature,
            message,
            &tree,
            &[&main_commit],
        )
        .suggestion(
            "You cannot provide an existing branch name. Choose a new branch name or run with '--force'.",
        )?; // TODO: How do I make sure this suggestion only gets shown if ErrorClass==Object and ErrorCode==-15?
    log::debug!(
        "Wrote quickfixed changes to new commit {} and new branch {}",
        commit_oid,
        target_branch
    );

    Ok(())
}

/// Removes the last commit from the current branch.
fn remove_commit_from_head(repo: &Repository) -> Result<(), Report> {
    // Equivalent to git reset --hard HEAD~1
    let head_1 = repo.head()?.peel_to_commit()?.parent(0)?;
    repo.reset(head_1.as_object(), ResetType::Hard, None)?;

    Ok(())
}

/// Pushes <branch> as new branch to `origin`. Other remote names are currently
/// not supported. If there is a need, please let us know.
pub fn push_new_commit(_repo: &Repository, branch: &str) -> Result<(), Report> {
    // TODO: Use git2 instead of Command.
    log::info!("Pushing new branch to origin.");
    let status = Command::new("git")
        .args(&["push", "--set-upstream", "origin", branch])
        .status()?;
    if !status.success() {
        eyre!("Failed to run git push. {}", status);
    } else {
        log::info!("Git push succeeded");
    }

    Ok(())
}

/// Checks that repo is in "RepositoryState::Clean" state. This means there is
/// no rebase, cherry-pick, merge, etc is in progress. Confusingly, this is different
/// from no uncommitted or staged changes being present in the repo. For this,
/// see [fn.assure_workspace_is_clean].
pub fn assure_repo_in_normal_state(repo: &Repository) -> Result<()> {
    let state = repo.state();
    if state != RepositoryState::Clean {
        return Err(eyre!(
            "The repository is currently not in a clean state ({:?}).",
            state
        ));
    }

    Ok(())
}

/// Checks that the workspace is clean. (No staged or unstaged changes.)
fn assure_workspace_is_clean(repo: &Repository) -> Result<()> {
    let mut options = git2::StatusOptions::new();
    options.include_ignored(false);
    let statuses = repo.statuses(Some(&mut options))?;
    for s in statuses.iter() {
        log::warn!("Dirty: {:?} -- {:?}", s.path(), s.status());
    }
    let is_dirty = !statuses.is_empty();
    if is_dirty {
        Err(eyre!("The repository is dirty."))
    } else {
        Ok(())
    }
}

/// A hacky way to resolve the default branch name on the 'origin' remote.
pub fn get_default_branch(repo: &Repository) -> Result<String, Report> {
    // NOTE: Unfortunately, I cannot use repo.find_remote().default_branch() because it requires a connect() before.
    // Furthermore, a lot is to be said about returning a Reference or a Revspec instead of a String.
    for name in [
        "origin/main",
        "origin/master",
        "origin/devel",
        "origin/develop",
    ]
    .iter()
    {
        match repo.resolve_reference_from_short_name(name) {
            Ok(_) => {
                log::debug!("Found {} as the default remote branch. A bit hacky -- wrong results certainly possible.", name);
                return Ok(name.to_string());
            }
            Err(_) => continue,
        }
    }
    Err(eyre!("Could not find remote default branch."))
}

/// Returns Ok(true) if stashing was successful. Ok(false) if stashing was not needed.
pub fn stash(repo: &mut Repository) -> Result<bool> {
    let signature = repo.signature()?;
    // Apologies for this code. This is just a fancy way of filtering out the (Stash, NotFound) error.
    let stashed = match repo.stash_save(&signature, "quickfix: auto-stash", None) {
        Ok(stash) => {
            log::debug!("Stashed to object {}", stash);
            true
        }
        Err(e) => {
            // Accept if there is nothing to stash.
            if e.code() == git2::ErrorCode::NotFound && e.class() == git2::ErrorClass::Stash {
                log::debug!("Nothing to stash.");
                false
            } else {
                return Err(eyre!("{}", e.message()));
            }
        }
    };

    Ok(stashed)
}
