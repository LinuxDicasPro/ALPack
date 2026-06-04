//! Utility functions for ALPack.
//!
//! Provides helper methods for path manipulation, environment discovery,
//! file downloads, and stylized terminal output.

use recursive_copy::{CopyOptions, copy_all};
use sandbox_utils::{
    SEPARATOR, SandBox, SandBoxConfig, app_name, failed_exist_rootfs, get_cmd_box, map_result,
};
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

/// Verifies that the specified rootfs directory exists and is accessible.
///
/// # Parameters
/// - `path`: The directory path to verify.
///
/// # Returns
/// - `Ok(())` if the directory exists.
/// - `Err` with diagnostic info if missing.
pub fn check_rootfs_exists(path: PathBuf) -> Result<(), Box<dyn Error>> {
    if !path.is_dir() {
        return failed_exist_rootfs(
            &format!("{} setup", app_name()),
            &path.display().to_string(),
        );
    }
    Ok(())
}

/// Matches packages against the database content and prints a standardized result box.
///
/// This function internalizes the search logic by invoking the `collect_matches!` macro.
/// It aggregates results from the provided database content based on the given package keys.
///
/// # Parameters
/// - `pkgs`: A slice of strings containing the package names or patterns to search for.
/// - `content`: The raw string content of the database file to be scanned.
///
/// # Returns
/// - `Ok(())` if matches were found and successfully printed to stdout.
/// - `Err` if the search result is empty or if the UI box generation fails.
pub fn print_result(pkgs: &[String], content: &str, generic: bool) -> Result<(), Box<dyn Error>> {
    let all_matches = collect_matches(pkgs, content, !generic);

    if all_matches.is_empty() {
        return Err(format!("{u}\nResult not found!\n{u}", u = SEPARATOR).into());
    }

    println!(
        "{u}\n{}\n{}\n{u}",
        get_cmd_box("SEARCH RESULT:", None, Some(18))?,
        all_matches.join("\n"),
        u = SEPARATOR
    );

    Ok(())
}

/// Sets up a local repository database within the rootfs.
///
/// This function ensures the build directory exists, clones the remote
/// repository using a blobless filter (`tree:0`) to save bandwidth, and
/// generates a flattened database file by filtering specific branches.
///
/// # Parameters
/// - `rootfs_dir`: Path to the root filesystem host directory.
/// - `url`: The remote Git repository URL.
/// - `repo`: The local name for the repository (e.g., "aports").
/// - `branches`: A list of branch names or paths to include in the database.
///
/// # Returns
/// - `Ok(())` if the repository was successfully initialized and indexed.
/// - `Err` if Git operations or filesystem modifications fail.
pub fn update_git_repository(
    profile: String,
    rootfs: PathBuf,
    url: &str,
    repo: &str,
    branches: &[&str],
) -> Result<(), Box<dyn Error>> {
    let build_dir = rootfs.join("build");
    let build_path = build_dir.join(repo);
    let database_path = build_dir.join(format!("{repo}-database"));

    let _ = fs::remove_dir_all(&build_path);
    let _ = fs::remove_file(&database_path);

    fs::create_dir_all(&build_path)?;

    let filter = branches.join("|");
    let run_cmd = format!(
        "type git > /dev/null || apk add git
        cd {}
        git clone --depth=1 --filter=tree:0 --no-checkout {url} {repo} && \
        cd {repo} && \
        git fetch --depth=1 --filter=tree:0 && \
        git ls-tree -r HEAD --name-only | grep -E \"({filter})\" > ../{repo}-database",
        build_dir.display(),
    );

    let config = SandBoxConfig {
        rootfs,
        profile,
        run_cmd,
        use_root: true,
        secure_rootfs: true,
        ..Default::default()
    };

    map_result(SandBox::run(config))?;
    Ok(())
}

/// Orchestrates the selective retrieval of package sources from a git repository.
///
/// It processes match results to identify relevant package directories,
/// configures Git's sparse-checkout to download only those specific paths,
/// and copies the resulting files to the final output destination.
///
/// # Parameters
/// - `rootfs`: Path to the root filesystem host directory.
/// - `repo_name`: The subdirectory name within `/build/` (e.g., "aports").
/// - `pkgs`: A slice of strings containing the package names to be retrieved.
/// - `content`: The raw string content of the database file.
/// - `output`: The destination directory for the retrieved files.
///
/// # Returns
/// - `Ok(())` if all package files were retrieved and copied.
/// - `Err` if no matches are found or if the sparse-checkout process fails.
pub fn download_git_sources_files(
    profile: String,
    rootfs: PathBuf,
    repo_name: &str,
    pkgs: &[String],
    content: &str,
    output: PathBuf,
) -> Result<(), Box<dyn Error>> {
    let repo_path = &rootfs.join("build").join(repo_name);
    let matches = collect_matches(pkgs, content, true);

    if matches.is_empty() {
        return Err(format!("{u}\nResult not found!\n{u}", u = SEPARATOR).into());
    }

    let pkg_dirs: HashSet<&str> = matches
        .iter()
        .filter(|line| line.contains("APKBUILD"))
        .filter_map(|line| line.rsplit_once('/').map(|(dir, _)| dir))
        .collect();

    let pkg_dirs_vec: Vec<&str> = pkg_dirs.into_iter().collect();

    let run_cmd = format!(
        "cd {}
        git sparse-checkout init --cone && \
        git sparse-checkout set {} && \
        git checkout",
        repo_path.display(),
        pkg_dirs_vec.join(" "),
    );

    let config = SandBoxConfig {
        rootfs: rootfs.clone(),
        profile,
        run_cmd,
        use_root: true,
        secure_rootfs: true,
        ..Default::default()
    };

    map_result(SandBox::run(config))?;

    let options = CopyOptions {
        overwrite: true,
        follow_symlinks: true,
        ..Default::default()
    };

    for dir in pkg_dirs_vec {
        copy_all(&repo_path.join(dir), &output, &options)?;
    }
    Ok(())
}

/// Collects unique lines from the database content that match the given terms.
///
/// When `scoped` is `true`, each term is wrapped in `/{term}/` to match only
/// lines within a specific package directory structure. When `false`, the term
/// is matched literally, useful for broader discovery searches.
///
/// # Parameters
/// * `terms`: A slice of `String` containing the search terms.
/// * `content`: The raw string content of the database file to be scanned.
/// * `scoped`: If `true`, wraps each term as `/{term}/` for exact package matching.
///             If `false`, matches any line containing the term literally.
///
/// # Returns
/// A sorted `Vec<&str>` of unique matching lines, where each element is a
/// reference to a slice of the original `content`, avoiding extra allocations.
fn collect_matches<'a>(terms: &[String], content: &'a str, unique: bool) -> Vec<&'a str> {
    let matches: HashSet<&str> = terms
        .iter()
        .flat_map(|term| {
            let pattern = if unique {
                format!("/{}/", term)
            } else {
                term.clone()
            };
            content.lines().filter(move |line| line.contains(&pattern))
        })
        .collect();

    let mut result: Vec<&str> = matches.into_iter().collect();
    result.sort();
    result.dedup();
    result
}
