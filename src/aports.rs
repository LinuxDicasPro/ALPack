//! Alpine Package Ports (aports) management module.
//!
//! This module provides the `Aports` struct and logic to interact with the
//! Alpine Linux aports repository, allowing for database updates,
//! package searching, and source file retrieval via sparse-checkout.

use crate::help::HELP_RULES;
use crate::settings::{settings_output_dir, settings_profile, settings_rootfs_dir};
use crate::utils;
use flexiargs::{Arg, ParserOptions, parse_into_vars};
use sandbox_utils::app_name;
use std::collections::VecDeque;
use std::error::Error;
use std::fs;

/// Controller for Alpine Linux repository operations.
pub struct Aports {
    /// Arguments passed from the CLI for processing.
    args: Vec<String>,
}

impl Aports {
    /// Creates a new `Aports` instance with the given context and arguments.
    pub fn new(args: Vec<String>) -> Self {
        Self { args }
    }

    /// Executes the aports command logic based on the provided arguments.
    ///
    /// The flow includes parsing arguments, optionally updating the local
    /// repository index, and performing search or fetch operations.
    ///
    /// # Performance
    /// - Uses `VecDeque<&str>` to avoid heap allocations during argument parsing.
    /// - Implements lazy loading for the database content.
    ///
    /// # Returns
    /// - `Ok(())` on success.
    /// - `Err` if argument validation, repository setup, or file operations fail.
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let args: VecDeque<String> = self.args.iter().cloned().collect();
        let mut rootfs_dir = settings_rootfs_dir();
        let mut output_dir = settings_output_dir();
        let mut profile = settings_profile();
        let (mut ss_pkg, mut s_pkg, mut get_pkg) = (Vec::new(), Vec::new(), Vec::new());
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
            subcommand: "aports",
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
                "https://github.com/alpinelinux/aports.git",
                "aports",
                &["main", "community", "testing"],
            )?;
        }

        if s_pkg.is_empty() && ss_pkg.is_empty() && get_pkg.is_empty() {
            return Ok(());
        }

        let db_path = rootfs_dir.join("build/aports-database");

        if !db_path.exists() {
            return Err(format!(
                "The aports database was not found at: {}\nPlease run '{} aports -u' first to initialize the repository.",
                db_path.display(), app_name()
            ).into());
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
                profile, rootfs_dir, "aports", &get_pkg, &content, output_dir,
            )?;
        }
        Ok(())
    }
}
