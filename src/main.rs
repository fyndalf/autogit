use git2::Repository;
use git2::RepositoryState;
use indicatif::{ProgressBar, ProgressStyle};
use quicli::prelude::*;
use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use structopt::StructOpt;

// Used for nice error messages
#[macro_use]
extern crate human_panic;

// A wrapper abstracting git operations
mod git_wrapper;

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
    println!("Updated {} repositories", update_count);
    Ok(())
}

// Recurse through subdirectories up until given maximal depth
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
                        trace!("{:?}", e); // errors are expected when folder is not a git directory
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

// Check if a folder contains a repository, without uncommited changes, and fetches from origin.
// Fails if the directory does not contain a git directory.
fn check_if_repo_is_clean(dir: &PathBuf, progress_bar: &ProgressBar) -> Result<bool, git2::Error> {
    let repo = Repository::open(dir)?;
    progress_bar.set_message(&format!("Checking {}", repo.path().display()));

    let branch_name = git_wrapper::get_current_branch(&repo)?;
    progress_bar.set_prefix(&format!("{} origin/{}", repo.path().display(), branch_name));

    debug!(
        "Checking {} state={:?}",
        repo.path().display(),
        repo.state()
    );

    progress_bar.set_message(&format!("Fetching origin/{}", branch_name));
    git_wrapper::fetch_origin(&repo, &branch_name)?;
    let files_changed = git_wrapper::get_diff_size(&repo);
    let cached_files_changed = git_wrapper::get_cached_diff_size(&repo);

    trace!(
        "Number of changed files:{:?}, number of changed cached files: {:?}",
        files_changed,
        cached_files_changed
    );
    Ok(repo.state() == RepositoryState::Clean && files_changed == Ok(0))
}

// Update a git repo on a given path. Discards changes when force is enabled.
fn update_repo(
    dir: &PathBuf,
    force_update: bool,
    progress_bar: &ProgressBar,
    update_count: &mut u16,
) -> Result<(), git2::Error> {
    let repo = Repository::open(dir)?;
    let branch_name = git_wrapper::get_current_branch(&repo)?;

    progress_bar.set_message("Updating");
    progress_bar.set_prefix(&format!("{} origin/{}", repo.path().display(), branch_name));

    if force_update {
        git_wrapper::reset_branch_to_remote(&repo, &branch_name)?;
    }

    git_wrapper::pull_branch_from_remote(&repo)?;
    *update_count += 1;
    Ok(())
}
