// A module wrapping git2 operations
use git2::Repository;
use std::env;

// Gets the name of the currently checked out branch. Defaults to master.
pub fn get_current_branch(repo: &Repository) -> Result<String, git2::Error> {
    let head = repo.head()?;
    let mut path: Vec<&str> = head.name().unwrap().split('/').collect();
    let branch = path.pop();
    let branch_name = match branch {
        None => "master",
        Some(_) => branch.unwrap(),
    };
    Ok(branch_name.to_string())
}

// callback function for git credentials
fn git_credentials_callback(
    _user: &str,
    _user_from_url: Option<&str>,
    _cred: git2::CredentialType,
) -> Result<git2::Cred, git2::Error> {
    let user = _user_from_url.unwrap_or("git");

    if _cred.contains(git2::CredentialType::USERNAME) {
        return git2::Cred::username(user);
    }

    match env::var("GPM_SSH_KEY") {
        Ok(k) => {
            println!(
                "authenticate with user {} and private key located in {}",
                user, k
            );
            git2::Cred::ssh_key(user, None, std::path::Path::new(&k), None)
        }
        _ => Err(git2::Error::from_str(
            "unable to get private key from GPM_SSH_KEY",
        )),
    }
}

// fetches the branch from origin
pub fn fetch_origin(repo: &Repository, branch_name: &str) -> Result<(), git2::Error> {
    let mut remote_callbacks = git2::RemoteCallbacks::new();
    remote_callbacks.credentials(git_credentials_callback);
    let mut fetch_opts = git2::FetchOptions::new();
    fetch_opts.remote_callbacks(remote_callbacks);
    fetch_opts.download_tags(git2::AutotagOption::All);

    repo.find_remote("origin")?
        .fetch(&[branch_name], Some(&mut fetch_opts), None)?;
    Ok(())
}

// gets the diff in a repository
pub fn get_diff_size(repo: &Repository) -> Result<usize, git2::Error> {
    let diff = repo.diff_index_to_workdir(None, None)?;
    let files_changed = diff.stats()?.files_changed();
    Ok(files_changed)
}

// gets the cached diff in a repository
pub fn get_cached_diff_size(repo: &Repository) -> Result<usize, git2::Error> {
    let cached_diff = repo.diff_tree_to_index(None, None, None)?;
    let cached_files_changed = cached_diff.stats()?.files_changed();
    Ok(cached_files_changed)
}

pub fn reset_branch_to_remote(repo: &Repository, branch_name: &str) -> Result<(), git2::Error> {
    let _head = repo.head()?;
    let ref_name = format!("refs/remotes/origin/{}", branch_name);
    let oid = repo.refname_to_id(&ref_name)?;
    let object = repo.find_object(oid, None).unwrap();
    repo.reset(&object, git2::ResetType::Hard, None)?;
    Ok(())
}

pub fn pull_branch_from_remote(repo: &Repository) -> Result<(), git2::Error> {
    let reference = repo.find_reference("FETCH_HEAD")?;
    let fetch_head_commit = repo.reference_to_annotated_commit(&reference)?;
    repo.merge(&[&fetch_head_commit], None, None)?;
    Ok(())
}
