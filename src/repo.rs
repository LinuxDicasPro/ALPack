//! Generic repository management module for ALPack.
//!
//! Provides a unified abstraction for interacting with Alpine-based package
//! repositories (e.g., aports, aptree). Handles database synchronization,
//! package searching, and source file retrieval via Git sparse-checkout.
//!
//! New repositories can be supported by defining a [`RepoConfig`] constant
//! and a thin wrapper struct.

use crate::help::HELP_RULES;
use crate::settings::{settings_output_dir, settings_profile, settings_rootfs_dir};
use crate::utils;
use flexiargs::{Arg, ParserOptions, parse_into_vars};
use sandbox_utils::app_name;
use std::collections::VecDeque;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

/// Static configuration describing a remote package repository.
///
/// Each repository (e.g., aports, aptree) defines its own `RepoConfig`
/// constant, which is passed to [`RepoCommand`] to drive the shared logic.
pub struct RepoConfig {
    /// The CLI subcommand name used to invoke this repository (e.g., `"aports"`).
    pub subcommand: &'static str,
    /// The remote Git URL to clone or update from.
    pub url: &'static str,
    /// The local directory name used under `/build/` for the clone and database
    /// (e.g., `"aports"` produces `/build/aports` and `/build/aports-database`).
    pub repo_name: &'static str,
    /// The list of branch paths to include when generating the flat database index.
    pub branches: &'static [&'static str],
}

/// Configuration for the Alpine Linux aports repository.
pub const APORTS_CONFIG: RepoConfig = RepoConfig {
    subcommand: "aports",
    url: "https://github.com/alpinelinux/aports.git",
    repo_name: "aports",
    branches: &["main", "community", "testing"],
};

/// Configuration for the Adélie Linux aptree repository.
pub const APTREE_CONFIG: RepoConfig = RepoConfig {
    subcommand: "aptree",
    url: "https://git.adelielinux.org/adelie/packages.git",
    repo_name: "aptree",
    branches: &["bootstrap", "experimental", "legacy", "system", "user"],
};

/// Shared command executor for repository operations.
///
/// Drives the full lifecycle of a repository command: argument parsing,
/// optional database update, package searching, and source retrieval.
/// Behavior is entirely determined by the [`RepoConfig`] provided at construction.
pub struct RepoCommand {
    /// Raw CLI arguments forwarded from the parent command.
    args: Vec<String>,
    /// Static configuration describing the target repository.
    config: RepoConfig,
}

impl RepoCommand {
    /// Creates a new `RepoCommand` with the given arguments and repository configuration.
    ///
    /// # Parameters
    /// * `args` - Raw CLI arguments to be parsed by this command.
    /// * `config` - A [`RepoConfig`] describing the target repository.
    ///
    /// # Returns
    /// A new `RepoCommand` instance ready to be executed via [`run`](Self::run).
    pub fn new(args: Vec<String>, config: RepoConfig) -> Self {
        Self { args, config }
    }

    /// Executes the repository command based on the parsed arguments.
    ///
    /// The execution flow is:
    /// 1. Parse CLI flags and collect package lists.
    /// 2. Verify the rootfs directory exists.
    /// 3. Optionally update the local Git database index.
    /// 4. Load the database and run the requested search or fetch operations.
    ///
    /// # Returns
    /// - `Ok(())` on success.
    /// - `Err` if argument validation, rootfs checks, database access,
    ///   or any Git or filesystem operation fails.
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let args: VecDeque<String> = self.args.iter().cloned().collect();
        let mut rootfs_dir = settings_rootfs_dir();
        let mut output_dir = settings_output_dir();
        let mut profile = settings_profile();
        let mut ss_pkg = Vec::new();
        let mut s_pkg = Vec::new();
        let mut get_pkg = Vec::new();
        let mut update = false;

        let mut rules = [
            Arg::bool(Some("-u"), "--update", &mut update).essential(),
            Arg::collect_list(Some("-S"), "--strict-search", "package", &mut ss_pkg).essential(),
            Arg::collect_list(Some("-s"), "--search", "package", &mut s_pkg).essential(),
            Arg::collect_list(Some("-g"), "--get", "package", &mut get_pkg).essential(),
            Arg::value(Some("-o"), "--output", "directory", &mut output_dir),
            Arg::value(Some("-R"), "--rootfs", "directory", &mut rootfs_dir),
            Arg::value(Some("-p"), "--profile", "profile", &mut profile),
        ];

        let opts = ParserOptions {
            subcommand: self.config.subcommand,
            help_rules: HELP_RULES,
            require_args: true,
            ..Default::default()
        };

        if parse_into_vars(&mut rules, args, opts).help_or_err()? {
            return Ok(());
        }

        drop(rules);

        utils::check_rootfs_exists(rootfs_dir.clone())?;

        if update {
            utils::update_git_repository(
                profile.clone(),
                rootfs_dir.clone(),
                self.config.url,
                self.config.repo_name,
                self.config.branches,
            )?;
        }

        if s_pkg.is_empty() && ss_pkg.is_empty() && get_pkg.is_empty() {
            return Ok(());
        }

        let db_path = self.db_path(&rootfs_dir);

        if !db_path.exists() {
            return Err(format!(
                "The {} database was not found at: {}\nPlease run '{} {} -u' first to initialize the repository.",
                self.config.repo_name,
                db_path.display(),
                app_name(),
                self.config.subcommand,
            )
                .into());
        }

        let content = fs::read_to_string(&db_path)?;

        if !s_pkg.is_empty() {
            utils::print_result(&s_pkg, &content, true)?;
        }

        if !ss_pkg.is_empty() {
            utils::print_result(&ss_pkg, &content, false)?;
        }

        if !get_pkg.is_empty() {
            utils::download_git_sources_files(
                profile,
                rootfs_dir,
                self.config.repo_name,
                &get_pkg,
                &content,
                output_dir,
            )?;
        }

        Ok(())
    }

    /// Returns the path to the flat database index file for this repository.
    ///
    /// The database is stored as `<rootfs>/build/<repo_name>-database`.
    ///
    /// # Parameters
    /// * `rootfs` - The root filesystem directory.
    ///
    /// # Returns
    /// A [`PathBuf`] pointing to the expected database file location.
    fn db_path(&self, rootfs: &PathBuf) -> PathBuf {
        rootfs.join(format!("build/{}-database", self.config.repo_name))
    }
}

/// Controller for Alpine Linux aports repository operations.
///
/// A thin wrapper around [`RepoCommand`] pre-configured with [`APORTS_CONFIG`].
pub struct Aports(RepoCommand);

impl Aports {
    /// Creates a new `Aports` instance with the given CLI arguments.
    ///
    /// # Parameters
    /// * `args` - Raw CLI arguments forwarded from the dispatcher.
    ///
    /// # Returns
    /// A new `Aports` instance ready to run.
    pub fn new(args: Vec<String>) -> Self {
        Self(RepoCommand::new(args, APORTS_CONFIG))
    }

    /// Executes the aports command.
    ///
    /// # Returns
    /// - `Ok(())` on success.
    /// - `Err` if any operation fails.
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        self.0.run()
    }
}

/// Controller for Adélie Linux aptree repository operations.
///
/// A thin wrapper around [`RepoCommand`] pre-configured with [`APTREE_CONFIG`].
pub struct Aptree(RepoCommand);

impl Aptree {
    /// Creates a new `Aptree` instance with the given CLI arguments.
    ///
    /// # Parameters
    /// * `args` - Raw CLI arguments forwarded from the dispatcher.
    ///
    /// # Returns
    /// A new `Aptree` instance ready to run.
    pub fn new(args: Vec<String>) -> Self {
        Self(RepoCommand::new(args, APTREE_CONFIG))
    }

    /// Executes the aptree command.
    ///
    /// # Returns
    /// - `Ok(())` on success.
    /// - `Err` if any operation fails.
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        self.0.run()
    }
}
