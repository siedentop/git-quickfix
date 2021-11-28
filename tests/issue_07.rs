// https://github.com/siedentop/git-quickfix/issues/7

use std::fs::File;

use assert_cmd::Command;
use eyre::Result;
use std::io::prelude::*;
use tempdir::TempDir;

mod common;

pub fn setup() -> String {
    // let dir = TempDir::new("quickfix").unwrap();
    let path = "/tmp/gitquickfix";
    std::fs::create_dir_all(path).unwrap();

    let out = Command::new("git")
        .current_dir(path)
        .arg("init")
        .output()
        .unwrap();
    println!("Output: {:?}, {:?}", out, path);
    assert!(out.status.success(), "Failed to init git repo. {:?}", out);

    path.to_string()
}

#[test]
fn empty_commit_on_newly_created_branch() -> Result<()> {
    let dir = setup();
    let cmd = common::git_qf_binary().current_dir(dir.clone());

    let git = |args: &[&str]| {
        Command::new("git")
            .current_dir(dir.clone())
            .args(args)
            .output()
            .unwrap()
    };

    // Create three commits, with some contents.
    for (commit_message, filename, content) in [
        ("first commit", "file1.txt", b"Hello, world!\n"),
        ("second commit", "file1.txt", b"Hello, world!\n"),
        ("third commit", "file1.txt", b"Hello, world!\n"),
    ]
    .into_iter()
    {
        let mut file = File::create(format!("{}/{}", dir, filename))?;
        file.write_all(*content)?;

        let _ = git(&["add", filename])
            .status
            .success()
            .then(|| ())
            .unwrap();
        let _ = git(&["commit", "-m", commit_message])
            .status
            .success()
            .then(|| ())
            .unwrap();
    }

    Ok(())
}

// fn main() {
//     empty_commit_on_newly_created_branch();
//     println!("Hello");
// }
