use std::process;

use git2::{self, Repository, RepositoryState, ResetType};
use process::Command;

use color_eyre::{eyre::Report, eyre::Result, Section};
use eyre::eyre;
use structopt::StructOpt;

extern crate log;

fn main() -> Result<()> {
    color_eyre::install()?;
    //  Big and bloats the code.
    env_logger::init();

    run()?;
    Ok(())
}

fn run() -> Result<(), Report> {
    let opts = Opt::from_args();
    let mut repo = Repository::open_from_env()?;

    assure_repo_in_normal_state(&repo)?;

    // TODO: Make this an integration test.
    // assert:
    // * keep is true.
    // * OR: repo is clean.
    // *     OR: stash is enabled

    if opts.keep {
        cherrypick_commit_onto_new_branch(&repo, &opts)?;
    } else {
        if opts.stash {
            let stash = repo.stash_save(&repo.signature()?, "quickfix: auto-stash", None)?;
            log::debug!("Stashed to object {}", stash);
        }
        assure_workspace_is_clean(&repo)
            .suggestion("Consider auto-stashing your changes with --stash.")
            .suggestion("Running this again with RUST_LOG=debug provides more details.")?;
        cherrypick_commit_onto_new_branch(&repo, &opts)?;
        remove_commit_from_head(&mut repo)?;

        if opts.stash {
            // NOTE: It would be good to verify that the right stash is popped.
            repo.stash_pop(0, None)?;
        }
    }

    if opts.push {
        push_new_commit(&repo, &opts.branch)?;
    }

    Ok(())
}

/// This cherry-picks a commit onto a new branch crated from default branch.
/// A typical use case is when one wants to quickly create a fix without leaving
/// the current branch. The quickfix-commit will then be cherry-picked onto a new
/// branch based-off origin/main. The benefit of this tool is that slow, expensive
/// and disruptive switching of branches is avoided. Everything is done in-memory
/// with no checkout of files on the file system.
///
/// Usage:
///
/// 1. Commit the changes
/// 2. git quickfix <new-branch>. This will create a new branch from the default branch.
///     `--push` will directly push this to the `origin` remote.
///     `--keep` will keep the quickfix commit on the current branch.
/// 3. The changes will be removed from the current branch, unless `--keep` was given to quickfix.
///
/// Benefits: Quickly provide unrelated fixes without having to abandon the current branch and switching branches.

#[derive(Debug, StructOpt)]
#[structopt(
    name = "git-quickfix",
    about = "Apply patches directly to the main branch.",
    verbatim_doc_comment
)]
struct Opt {
    #[structopt(help = "The new branch name where the quickfix ends up on.")]
    branch: String,
    #[structopt(
        long = "push",
        short = "u",
        help = "Push the newly crated branch to origin."
    )]
    push: bool,
    #[structopt(
        long = "keep",
        short = "k",
        help = "Keep the new quickfix commit on the current branch."
    )]
    keep: bool,
    #[structopt(
        help = "The starting point onto which the quickfix gets applied. Defaults to the default branch on origin (e.g. origin/main).",
        long = "onto",
        short = "o"
    )]
    onto: Option<String>,
    #[structopt(
        help = "Overwrite the new branch, if it exists.",
        long = "force",
        short = "f"
    )]
    force: bool,
    #[structopt(
        help = "Automatically stash changes before modifying the current branch.",
        long = "stash",
        short = "s"
    )]
    stash: bool,
}

fn cherrypick_commit_onto_new_branch(repo: &Repository, opts: &Opt) -> Result<(), Report> {
    let onto_branch = match &opts.onto {
        Some(b) => b.clone(),
        None => {
            get_default_branch(&repo).suggestion("Manually set the target branch with `--onto`.")?
        }
    };

    let main_commit = repo
        .revparse(&onto_branch)?
        .from()
        .unwrap()
        .peel_to_commit()?;

    // Create the new branch
    let new_branch = repo
        .branch(&opts.branch, &main_commit, opts.force)
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
    let tree_oid = index.write_tree_to(&repo)?;
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
        opts.branch
    );

    Ok(())
}

/// Removes the last commit from the current branch.
fn remove_commit_from_head(repo: &mut Repository) -> Result<(), Report> {
    // Equivalent to git reset --hard HEAD~1
    let head_1 = repo.head()?.peel_to_commit()?.parent(0)?;
    repo.reset(&head_1.as_object(), ResetType::Hard, None)?;

    Ok(())
}

/// Pushes <branch> as new branch to `origin`. Other remote names are currently
/// not supported. If there is a need, please let us know.
fn push_new_commit(_repo: &Repository, branch: &str) -> Result<(), Report> {
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
/// see [fn.assure_repo_is_clean].
fn assure_repo_in_normal_state(repo: &Repository) -> Result<()> {
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
fn get_default_branch(repo: &Repository) -> Result<String, Report> {
    // NOTE: Unfortunately, I cannot use repo.find_remote().default_branch() because it requires a connect() before.
    // Furthermore, a lot is to be said about returning a Reference or a Revspec instead of a String.
    for name in ["origin/main", "origin/master", "origin/devel"].iter() {
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
