use assert_cmd::Command;
use structopt::clap::crate_name;

pub fn git_qf_binary() -> Command {
    Command::cargo_bin(crate_name!()).unwrap()
}
