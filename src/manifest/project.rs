use std::fmt;
use std::path::PathBuf;

use colored::*;
use failure::Error;
use git2;
use git2::{Repository, Status};
use itertools::Itertools;

#[derive(Debug, Deserialize)]
pub struct Project {
    pub name: String,
    pub path: Option<String>,
    pub groups: Option<String>,
    pub revision: Option<String>,
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

impl Project {
    pub fn get_status(self, repo_root: PathBuf) -> Result<String, Error> {
        let index_change: Status = Status::INDEX_NEW | Status::INDEX_MODIFIED
            | Status::INDEX_DELETED | Status::INDEX_RENAMED;
        let worktree_change = Status::WT_NEW | Status::WT_MODIFIED | Status::WT_DELETED
            | Status::WT_TYPECHANGE | Status::WT_RENAMED;

        let project_path = self.path.unwrap_or(self.name);
        let repo = Repository::init(repo_root.join(&project_path))?;
        let mut options = git2::StatusOptions::new();
        options.include_ignored(false);
        let statuses = repo.statuses(Some(&mut options))?
            .iter()
            .filter_map(|status| {
                if !status.status().intersects(index_change | worktree_change) {
                    return None;
                }

                let st = status.status();
                let line = format!(" {}\t\t{}", GitStatus(st), status.path().unwrap());
                if st.intersects(index_change) && !st.contains(worktree_change) {
                    Some(line.green())
                } else {
                    Some(line.red())
                }
            })
            .join("\n");
        if !statuses.is_empty() {
            Ok(format!("project {}/\n{}", project_path.bold(), statuses))
        } else {
            Ok(String::new())
        }
    }
}
