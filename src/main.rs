use git2::{self, Repository};
use git_quickfix::{
    assure_repo_in_normal_state, cherrypick_commit_onto_new_branch, get_default_branch,
    push_new_commit, stash, wrapper_pick_and_clean,
};

use color_eyre::{eyre::Report, eyre::Result, Section};
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

    let onto_branch = match &opts.onto {
        Some(b) => b.clone(),
        None => {
            get_default_branch(&repo).suggestion("Manually set the target branch with `--onto`.")?
        }
    };

    let target_branch = opts.branch;

    assure_repo_in_normal_state(&repo)?;

    // TODO: Make this an integration test.
    // assert:
    // * remove is false.
    // * OR: repo is clean.
    // *     OR: stash is enabled

    if opts.remove {
        let stashed = if opts.stash { stash(&mut repo)? } else { false };

        let result = wrapper_pick_and_clean(&repo, &target_branch, &onto_branch, opts.force);
        if stashed {
            repo.stash_pop(0, None)?
        };
        result?;
    } else {
        cherrypick_commit_onto_new_branch(&repo, &target_branch, &onto_branch, opts.force)?;
    }

    if opts.push {
        push_new_commit(&repo, &target_branch)?;
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
///     `--remove` will remove the quickfix commit from the current branch.
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
        long = "remove",
        help = "Remove the new quickfix commit from the current (original) branch."
    )]
    remove: bool,
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
        long = "autostash",
        short = "s"
    )]
    stash: bool,
}
