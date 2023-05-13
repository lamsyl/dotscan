use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{env, process::Command};

fn main() -> Result<()> {
    let current_dir = env::current_dir()?;
    let home_dir = dirs::home_dir().ok_or(anyhow!("Failed to get home dir"))?;

    let mut git_dir = PathBuf::from(&home_dir);
    git_dir.push("dotfiles");

    let work_tree = &home_dir;

    let current_dir_file_names =
        get_file_names(&current_dir).context("Failed to get file names in current dir")?;
    let tracked_file_names = get_tracked_file_names(&current_dir, &git_dir, work_tree)
        .context("Failed to get tracked file names")?;
    let current_dir_tracked = count_tracked_files(tracked_file_names);
    report_tracking_status(current_dir_file_names, &current_dir_tracked);

    Ok(())
}

fn get_file_names<P: AsRef<Path>>(dir: P) -> Result<impl Iterator<Item = String>> {
    let output = Command::new("ls")
        .args(["-p"]) // make dir end with "/"
        .current_dir(dir)
        .output()
        .context("Failed to get ls output")?;
    let stdout = String::from_utf8(output.stdout)?;
    let file_names: Vec<_> = stdout
        .split("\n")
        .filter_map(|s| {
            if s.len() > 0 {
                Some(String::from(s))
            } else {
                None
            }
        })
        .collect();
    Ok(file_names.into_iter())
}

fn get_tracked_file_names<P: AsRef<Path>>(
    dir: P,
    git_dir: P,
    work_tree: P,
) -> Result<impl Iterator<Item = String>> {
    let output = Command::new("git")
        .args([
            &format!("--git-dir={}", git_dir.as_ref().display()),
            &format!("--work-tree={}", work_tree.as_ref().display()),
            "ls-tree",
            "--name-only",
            "-r",
            "HEAD",
        ])
        .current_dir(dir)
        .output()
        .context("Failed to use git ls-tree to list tracked files")?;
    let stdout = String::from_utf8(output.stdout)?;
    let file_names: Vec<_> = stdout
        .split("\n")
        .filter_map(|s| {
            if s.len() > 0 {
                Some(String::from(s))
            } else {
                None
            }
        })
        .collect();
    Ok(file_names.into_iter())
}

// count tracked files when encounter a directory prefix
// count = 1 when encounter a bare file name
fn count_tracked_files<I>(file_names_iter: I) -> HashMap<String, u32>
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    let mut h = HashMap::<String, u32>::new();
    let mut increment_tracked = |dir_or_file: String| {
        h.entry(dir_or_file)
            .and_modify(|counter| *counter += 1)
            .or_insert(1);
    };

    for file_name in file_names_iter {
        let mut split_by_dir = file_name.as_ref().split("/");
        match (split_by_dir.next(), split_by_dir.next()) {
            (Some(dir_name), Some(_)) => increment_tracked(format!("{}/", dir_name)),
            (Some(file_name), None) => increment_tracked(String::from(file_name)),
            (None, _) => {
                unreachable!("split_by_dir should at least have one item")
            }
        }
    }

    h
}

// Pre-condition:
//   - all path in relative path without ./ prefix
//   - directory name ends with /
fn report_tracking_status<I>(file_names: I, tracked: &HashMap<String, u32>)
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    for file_name in file_names {
        let file_name = String::from(file_name.as_ref());
        if file_name.ends_with("/") {
            let dir_name = file_name;
            match tracked.get(&dir_name) {
                Some(&count) => report_dir_with_tracked_files(&dir_name, count),
                None => report_dir_without_tracked_files(&dir_name),
            }
        } else {
            match tracked.get(&file_name) {
                Some(&count) if count == 1 => report_tracked_file(&file_name),
                Some(_) => unreachable!("file should have track count of 1"),
                None => report_untracked_file(&file_name),
            }
        }
    }
}

fn report_dir_with_tracked_files(dir_name: &str, count: u32) {
    println!("{} - {}", dir_name, count);
}

fn report_dir_without_tracked_files(dir_name: &str) {
    println!("{} - None", dir_name);
}

fn report_tracked_file(file_name: &str) {
    println!("{} - CHECKED", file_name);
}

fn report_untracked_file(file_name: &str) {
    println!("{} - LEFT", file_name);
}

#[allow(dead_code)]
fn print_files<I>(iter: I)
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    for file_name in iter {
        println!("{:?}", file_name.as_ref());
    }
}
