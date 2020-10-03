use std::process;

use git2;
use git2::Repository;
use process::Command;

use color_eyre::{eyre::Report, eyre::Result, Section};
use eyre::eyre;
use structopt::StructOpt;

extern crate log;

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
    #[structopt(long = "force", short = "f", help = "Write over an existing branch.")]
    force: bool,
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

    // Commit current index to current branch.
    let author = repo.signature()?;
    let tree_oid = repo.index()?.write_tree()?;
    let tree = repo.find_tree(tree_oid)?;
    let parent_oid = repo.head()?.target().unwrap();
    let parent = repo.find_commit(parent_oid)?;
    let commit_oid = repo.commit(None, &author, &author, "temporary", &tree, &[&parent])?;
    let commit = repo.find_commit(commit_oid)?;

    let main_ref = repo
        .revparse(&opts.onto.unwrap())?
        .from()
        .unwrap()
        .peel_to_commit()?;
    let main_commit = repo.find_commit(main_ref.id())?;

    // Cherry-pick
    let mut index = repo.cherrypick_commit(&commit, &main_commit, 0, None)?;
    let tree_oid = index.write_tree_to(&repo)?;
    let tree = repo.find_tree(tree_oid)?;

    let message = commit
        .message_raw()
        .ok_or_else(|| eyre!("Could not read the commit message."))
        .suggestion("Make sure the commit message contains only UTF-8 characters or try to manually cherry-pick the commit.")?;

    // TODO: try update_ref as fully qualified.
    let commit_oid = repo.commit(None, &author, &author, message, &tree, &[&main_commit])?;
    let commit = repo.find_commit(commit_oid)?;

    let _branch = repo.branch(&opts.branch, &commit, opts.force)?;

    // TODO: Don't forget to clean up the index (still added)
    if opts.keep {
        // TODO: use the commit
    } else {
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
