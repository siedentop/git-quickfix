use std::process;

use git2::Repository;
use git2::{self, ResetType};
use process::Command;

use color_eyre::{eyre::Report, eyre::Result, Section};
use eyre::eyre;
use structopt::StructOpt;

extern crate log;

// TODO: How do I get the formatting to stay?
/// This applies a patch directly to the main branch.
///
/// Usage:
///
/// 1. `git add` as usual.
/// 2. git quickfix <new-branch>. This will create a new branch from the main branch.
///     `--push` will directly push this to the `origin` remote.
///     `--keep` will keep the patch on the current branch.
/// 3. The changes will not remain on the current branch, unless `--keep` was given to quickfix.
///
/// Benefits: Quickly provide unrelated fixes without having to abandon the current branch and switching branches.

#[derive(Debug, StructOpt)]
#[structopt(
    name = "git-quickfix",
    about = "Apply patches directly to the main branch."
)]
struct Opt {
    branch: String,
    #[structopt(short = "m", long = "message", help = "Commit message")]
    message: Option<String>,
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
        help = "The branch to apply the patch onto. Defaults to origin/main .",
        skip
    )]
    onto: Option<String>,
}

fn run() -> Result<(), Report> {
    let mut opts = Opt::from_args();
    opts.onto = Some("origin/master".to_string()); // TODO: This is planned to be defaulted to origin/main or origin/master if the first is not available.

    let repo = Repository::open_from_env()?;

    // Cherry-pick the HEAD onto the main branch but in memory.
    // Then create a new branch with that cherry-picked commit.
    let fix_commit = repo.head()?.peel_to_commit()?;

    let main_commit = repo
        .revparse(&opts.onto.unwrap())?
        .from()
        .unwrap()
        .peel_to_commit()?;

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

    // TODO: try update_ref as fully qualified.
    let commit_oid = repo
        .commit(
            Some(&format!("refs/heads/{}", opts.branch)),
            &fix_commit.author(),
            &signature,
            message,
            &tree,
            &[&main_commit],
        )
        .suggestion(
            "You cannot provide an existing branch name. Choose a new branch name or run with.",
        )?; // TODO: How do I make sure this suggestion only gets shown if ErrorClass==Object and ErrorCode==-15?
    log::debug!(
        "Wrote quickfixed changes to new commit {} and new branch {}",
        commit_oid,
        opts.branch
    );

    // TODO: What to do if the index or working dir is dirty?
    if !opts.keep {
        // Equivalent to git reset --hard HEAD~1
        if fix_commit.parent_count() != 1 {
            return Err(eyre!("Only works with non-merge commits"))
                .suggestion("Quickfixing a merge commit is not supported. If you meant to do this please file a ticket with your usecase.");
        };
        let head_1 = fix_commit.parent(0)?;
        repo.reset(&head_1.as_object(), ResetType::Hard, None)?;
    }

    // TODO: Use git2 instead of Command.
    if opts.push {
        log::info!("Pushing new branch to origin.");
        let status = Command::new("git")
            .args(&["push", "--set-upstream", "origin", &opts.branch])
            .status()?;
        if !status.success() {
            log::error!("Failed to run git push. {}", status);
        } else {
            log::info!("Git push succeeded");
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    color_eyre::install()?;
    //  Big and bloats the code.
    env_logger::init();

    run()?;
    Ok(())
}
