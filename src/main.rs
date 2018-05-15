extern crate colored;
extern crate git2;
extern crate itertools;
extern crate xml;

use colored::*;
use git2::{Repository, Status};
use xml::reader::{EventReader, XmlEvent};

use itertools::Itertools;
use std::env;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

fn find_repo_root() -> Option<PathBuf> {
    let mut path = env::current_dir().ok()?;
    loop {
        let repo_path = path.join(".repo");
        if repo_path.exists() && repo_path.is_dir() {
            return Some(path);
        }
        path = PathBuf::from(path.parent()?);
    }
}

fn projects(manifest: &Path) -> impl Iterator<Item = String> {
    let file = File::open(manifest).unwrap();
    let file = BufReader::new(file);

    let parser = EventReader::new(file);
    parser.into_iter().filter_map(|e| match e {
        Ok(XmlEvent::StartElement {
            name, attributes, ..
        }) => if name.local_name == "project" {
            for attr in attributes {
                if attr.name.local_name == "path" {
                    return Some(attr.value);
                }
            }
            None
        } else {
            None
        },
        Err(e) => {
            // TODO: stop gracefully
            panic!("Error: {}", e);
        }
        _ => None,
    })
}

struct GitStatus(Status);

impl fmt::Display for GitStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let st = self.0;
        let index_flag = if st.contains(Status::INDEX_NEW) {
            'A'
        } else if st.contains(Status::INDEX_MODIFIED) {
            'M'
        } else if st.contains(Status::INDEX_DELETED) {
            'D'
        } else if st.contains(Status::INDEX_RENAMED) {
            'R'
        } else {
            '-'
        };
        let worktree_flag = if st.contains(Status::WT_NEW) {
            'a'
        } else if st.contains(Status::WT_MODIFIED) {
            'm'
        } else if st.contains(Status::WT_DELETED) {
            'd'
        } else if st.contains(Status::WT_TYPECHANGE) {
            't'
        } else if st.contains(Status::WT_RENAMED) {
            'r'
        } else {
            '-'
        };
        write!(f, "{}{}", index_flag, worktree_flag)
    }
}

fn main() -> Result<(), Box<Error>> {
    let index_change: Status =
        Status::INDEX_NEW | Status::INDEX_MODIFIED | Status::INDEX_DELETED | Status::INDEX_RENAMED;
    let worktree_change = Status::WT_NEW | Status::WT_MODIFIED | Status::WT_DELETED
        | Status::WT_TYPECHANGE | Status::WT_RENAMED;

    let repo_root = match find_repo_root() {
        Some(p) => p,
        None => panic!("Cannot find root of repo."),
    };

    for path in projects(&repo_root.join(".repo/manifest.xml")) {
        let repo = Repository::init(repo_root.join(&path))?;
        let statuses = repo.statuses(None)?
            .iter()
            .filter_map(|status| {
                if !status.status().intersects(index_change | worktree_change) {
                    return None;
                }

                let st = status.status();
                let line = format!(" {}     {}", GitStatus(st), status.path().unwrap());
                if st.intersects(index_change) && !st.contains(worktree_change) {
                    Some(line.green())
                } else {
                    Some(line.red())
                }
            })
            .join(",");
        if !statuses.is_empty() {
            println!("project {}/\n{}", path.bold(), statuses);
        }
    }
    Ok(())
}
