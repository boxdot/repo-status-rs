extern crate clap;
extern crate colored;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate git2;
extern crate itertools;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_xml_rs;

use std::env;
use std::path::PathBuf;
use std::process::Command;

use clap::{App, SubCommand};
use colored::*;
use failure::Error;
use futures::executor::ThreadPool;
use futures::future;
use futures::prelude::*;
use itertools::Itertools;

mod manifest;
use manifest::Manifest;

#[derive(Debug, Fail)]
enum RepoError {
    #[fail(display = ".repo not found in the directory tree.")]
    RepoRootNotFound,
}

fn find_repo_root() -> Result<PathBuf, Error> {
    let mut path = env::current_dir()?;
    loop {
        let repo_path = path.join(".repo");
        if repo_path.exists() && repo_path.is_dir() {
            return Ok(path);
        }
        path = PathBuf::from(path.parent().ok_or(RepoError::RepoRootNotFound)?);
    }
}

/// Launch the original `repo` with all the provided arguments
fn launch_repo() -> Result<i32, Error> {
    let return_code = Command::new("repo")
        .args(env::args_os().skip(1))
        .status()?
        .code()
        .ok_or(format_err!("repo subprocess exited without a return code."))?;
    ::std::process::exit(return_code);
}

fn run() -> Result<(), Error> {
    let matches = App::new("repo")
        .subcommand(SubCommand::with_name("status").help("Compares the working tree to the staging area (index) and the most recent commit on this branch (HEAD) in all"))
        .get_matches_safe().map_err(|_| launch_repo()).unwrap();

    if let Some(_matches) = matches.subcommand_matches("status") {
        let repo_root = find_repo_root()?;
        env::set_current_dir(repo_root)?;
        let manifest = Manifest::from_current_dir()?;
        let fut_output = future::join_all(
            manifest
                .projects
                .into_iter()
                .map(move |project| future::result(project.get_status())),
        ).and_then(|outputs: Vec<String>| {
            Ok(println!(
                "{}",
                outputs
                    .into_iter()
                    .filter(|line| !line.is_empty())
                    .join("\n")
            ))
        });

        ThreadPool::new()
            .expect("Failed to create threadpool")
            .run(fut_output)
    } else {
        Ok(())
    }
}

fn main() {
    if let Err(e) = run() {
        println!("{} {}", "Error:".red(), e);
        std::process::exit(1);
    }
}
