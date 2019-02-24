use console::{style, Emoji};
use git2::Repository;
use git2::RepositoryState;
use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressStyle};
use quicli::prelude::*;
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use structopt::StructOpt;

#[macro_use]
extern crate human_panic;

/// Update all git repositories that are located in subfolders
#[derive(Debug, StructOpt)]
struct Cli {
    /// How deep to check for git repositories
    #[structopt(long = "depth", short = "d", default_value = "3")]
    depth: usize,
    /// Force resetting and updating of currently checked out branches
    #[structopt(long = "force", short = "f")]
    force: bool,
}

static CHECK_BOX: Emoji = Emoji("✔", "");
static ERROR: Emoji = Emoji("x", "");

fn main() -> CliResult {
    setup_panic!();

    let args = Cli::from_args();

    println!(
        "Updating all git repositories up to a depth of {}",
        args.depth
    );
    let spinner_style = ProgressStyle::default_spinner()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
        .template("{prefix:.bold.dim} {spinner} {wide_msg}");
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(spinner_style);
    spinner.enable_steady_tick(100);

    let path = env::current_dir()?;
    let mut update_count: u16 = 0;

    visit_dirs(
        &path,
        1,
        args.depth,
        args.force,
        &spinner,
        &mut update_count,
    )?;
    spinner.set_prefix("");
    spinner.set_message("");
    spinner.finish_with_message("Finished updating");
    println!("{} Updated {} repositories", CHECK_BOX, update_count);
    Ok(())
}

fn visit_dirs(
    dir: &PathBuf,
    depth: usize,
    max_depth: usize,
    force_update: bool,
    progress_bar: &ProgressBar,
    update_count: &mut u16,
) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                // check git status and take actions, or recurse further
                trace!("Checking {:?}", fs::canonicalize(&path));
                match check_if_repo_is_clean(&path, progress_bar) {
                    Ok(clean_repo) => {
                        if clean_repo || force_update {
                            update_repo(&path, force_update, progress_bar, update_count)
                                .unwrap_or_else(|_| {
                                    panic!("Failed to update repo {:?}", fs::canonicalize(&path))
                                });
                        }
                    }
                    Err(e) => {
                        trace!("{} {:?}", ERROR, e);
                        if depth < max_depth {
                            visit_dirs(
                                &path,
                                depth + 1,
                                max_depth,
                                force_update,
                                progress_bar,
                                update_count,
                            )?;
                        }
                    }
                };
            }
        }
    }
    Ok(())
}

fn check_if_repo_is_clean(dir: &PathBuf, progress_bar: &ProgressBar) -> Result<bool, git2::Error> {
    let repo = Repository::open(dir)?;
    let branch_name = get_current_branch(&repo)?;
    let path = dir.as_os_str().to_str().unwrap();
    debug!(
        "Checking {} state={:?}",
        repo.path().display(),
        repo.state()
    );
    progress_bar.set_message(&format!("Checking {}", repo.path().display()));
    progress_bar.set_prefix(&format!("{} origin/{}", repo.path().display(), branch_name));

    // fetching branch
    progress_bar.set_message(&format!("Fetching origin/{}", branch_name));

    // repo.find_remote("origin")?.fetch(&[&branch_name], None, None)?;
    // todo: authenticate and use git2 instead of command
    let _output = Command::new("git")
        .current_dir(path)
        .arg("fetch")
        .output()
        .expect("Failed to execute git fetch");

    let diff = repo.diff_index_to_workdir(None, None)?;
    let files_changed = diff.stats()?.files_changed();

    let cached_diff = repo.diff_tree_to_index(None, None, None)?;
    let cached_files_changed = cached_diff.stats()?.files_changed();

    trace!(
        "Numer of changed files:{}, number of changed cached files: {}",
        files_changed,
        cached_files_changed
    );
    Ok(repo.state() == RepositoryState::Clean && files_changed == 0)
}

fn update_repo(
    dir: &PathBuf,
    force_update: bool,
    progress_bar: &ProgressBar,
    update_count: &mut u16,
) -> Result<(), git2::Error> {
    let repo = Repository::open(dir)?;
    let branch_name = get_current_branch(&repo)?;
    let path = dir.as_os_str().to_str().unwrap();

    progress_bar.set_message("Updating ...");
    progress_bar.set_prefix(&format!("{} origin/{}", repo.path().display(), branch_name));

    if force_update {
        let _head = repo.head()?;
        let ref_name = format!("refs/remotes/origin/{}", branch_name);
        let oid = repo.refname_to_id(&ref_name)?;
        let object = repo.find_object(oid, None).unwrap();
        repo.reset(&object, git2::ResetType::Hard, None)?;
    }

    debug!("Updating {:?} {}", fs::canonicalize(&dir), branch_name);

    let _output = Command::new("git")
        .current_dir(path)
        .arg("pull")
        .output()
        .expect("Failed to execute git pull");
    *update_count += 1;
    Ok(())
}

fn get_current_branch(repo: &Repository) -> Result<String, git2::Error> {
    let head = repo.head()?;
    let mut path: Vec<&str> = head.name().unwrap().split('/').collect();
    let branch = path.pop();
    let branch_name = match branch {
        None => "master",
        Some(_) => branch.unwrap(),
    };
    Ok(branch_name.to_string())
}
