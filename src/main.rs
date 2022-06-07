mod build_index;
mod common;

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use argh::FromArgs;
use std::process::Command;

use crate::{
    build_index::build_index_and_get_size,
    common::{check_output, exec_stream},
};

#[derive(FromArgs)]
/// Reach new heights.
struct Options {
    /// skip quickwit installation
    #[argh(option)]
    skip_quickwit_install: Option<bool>,

    /// the path to the config to build indices
    #[argh(option)]
    build_indices_config_path: Option<PathBuf>,

    /// the quickwit_commit_hash to checkout and build
    #[argh(option)]
    quickwit_commit_hash: Option<String>,

    /// the machine name, in case there are multiple machine executing into the same db
    #[argh(option)]
    machine_name: String,
}

fn main() -> std::io::Result<()> {
    let opt: Options = argh::from_env();

    if !opt.skip_quickwit_install.unwrap_or(false) {
        get_and_compile_quickwit(opt.quickwit_commit_hash)?;
        let quickwit = Path::new("../");
        assert!(env::set_current_dir(&quickwit).is_ok());
    }

    let output = Command::new("git")
        .current_dir("quickwit")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .expect("failed to execute process");

    let commit_hash = String::from_utf8(output.stdout)
        .expect("could not parse command output for get git commit hash");

    build_index_and_get_size(
        opt.build_indices_config_path,
        &opt.machine_name,
        &commit_hash,
    )?;

    Ok(())
}

fn get_and_compile_quickwit(quickwit_commit_hash: Option<String>) -> std::io::Result<()> {
    if Path::new("quickwit").exists() {
        fs::remove_dir_all("quickwit")?;
    }

    let output = Command::new("git")
        .args(["clone", "https://github.com/quickwit-oss/quickwit.git"])
        .output()
        .expect("failed to execute process");

    check_output(output);

    let quickwit = Path::new("./quickwit");
    assert!(env::set_current_dir(&quickwit).is_ok());

    if let Some(commit_hash) = quickwit_commit_hash {
        let output = Command::new("git")
            .args(["reset", "--hard", &commit_hash])
            .output()
            .expect("failed to execute process");

        check_output(output);
    }

    exec_stream("cargo", ["build", "--release"].as_ref());

    Ok(())
}
