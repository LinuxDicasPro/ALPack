//! # Overlay Subcommand Module
//!
//! This module provides functionality to manage and interact with OverlayFS
//! configurations, allowing users to layer filesystem changes over the rootfs.

use crate::help::HELP_RULES;
use crate::settings::{
    settings_overlay_action, settings_overlay_inode_mode, settings_rootfs_dir, settings_use_overlay,
};
use flexiargs::{Arg, ParserOptions, parse_into_vars};
use sandbox_utils::{
    AskConfig, CommitFilterConfig, DialogConfig, OverlayConfig, SandBoxConfig, UpperChoice,
    apply_choice, list_available_profiles, print_overlay_status, set_profile,
};
use std::collections::VecDeque;
use std::error::Error;
use std::path::PathBuf;

/// Manager for the `overlay` subcommand execution.
pub struct Overlay {
    /// Arguments captured after the `overlay` keyword.
    args: Vec<String>,
}

impl Overlay {
    /// Creates a new `Overlay` instance with the provided arguments.
    pub fn new(args: Vec<String>) -> Self {
        Self { args }
    }

    /// Executes the overlay filesystem operation using the configured arguments.
    ///
    /// When an action flag is provided (`--ask`, `--commit`, `--commit-atomic`,
    /// `--discard`), the corresponding operation is applied to the upper layer of
    /// the selected profile. `--no-confirm` skips the interactive prompt for
    /// non-ask actions. Commit filter flags allow fine-grained control over which
    /// files are included or excluded during the commit process. Without any action
    /// flag, the current overlay status and available profiles are printed.
    ///
    /// # Returns
    /// * `Ok(())` - If the operation completes successfully.
    /// * `Err` - If argument parsing, profile resolution, or the overlay operation fails.
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let args: VecDeque<String> = self.args.iter().cloned().collect();
        let mut rootfs = settings_rootfs_dir();
        let mut profile: Option<String> = None;
        let use_overlay = settings_use_overlay();
        let (mut no_confirm, mut do_ask, mut do_commit, mut do_commit_atomic, mut do_discard) =
            (false, false, false, false, false);
        let (mut skip_homedir, mut skip_rootdir, mut skip_zero_perms, mut no_rootfs_preset) =
            (false, false, false, false);
        let (mut force_files, mut skip_dirs, mut skip_files) = (Vec::new(), Vec::new(), Vec::new());
        let (mut skip_regex, mut skip_glob, mut skip_empty_in) =
            (Vec::new(), Vec::new(), Vec::new());

        let mut rules = [
            Arg::bool(None, "--ask", &mut do_ask),
            Arg::bool(None, "--commit", &mut do_commit),
            Arg::bool(None, "--commit-atomic", &mut do_commit_atomic),
            Arg::bool(None, "--discard", &mut do_discard),
            Arg::bool(None, "--skip-homedir", &mut skip_homedir),
            Arg::bool(None, "--skip-rootdir", &mut skip_rootdir),
            Arg::bool(None, "--skip-zero-perms", &mut skip_zero_perms),
            Arg::bool(None, "--no-rootfs-preset", &mut no_rootfs_preset),
            Arg::bool(None, "--no-confirm", &mut no_confirm),
            Arg::collect_list(None, "--force-files", "path", &mut force_files),
            Arg::collect_list(None, "--skip-dirs", "path", &mut skip_dirs),
            Arg::collect_list(None, "--skip-files", "name", &mut skip_files),
            Arg::collect_list(None, "--skip-regex", "pattern", &mut skip_regex),
            Arg::collect_list(None, "--skip-glob", "pattern", &mut skip_glob),
            Arg::collect_list(None, "--skip-empty-in", "path", &mut skip_empty_in),
            Arg::value(Some("-R"), "--rootfs", "directory", &mut rootfs),
            Arg::option(Some("-p"), "--set-profile", "profile", &mut profile),
        ];

        let opts = ParserOptions {
            subcommand: "overlay",
            help_rules: HELP_RULES,
            ..Default::default()
        };

        if parse_into_vars(&mut rules, args, opts).help_or_err()? {
            return Ok(());
        }

        drop(rules);

        let action_requested = do_ask || do_commit || do_commit_atomic || do_discard;

        if action_requested {
            let profile = profile.ok_or(
                "A profile must be specified with -p/--set-profile when using action flags.",
            )?;
            let profile_name = set_profile(&profile);
            let profiled_rootfs = rootfs.join(&profile_name);
            let upper = rootfs.join(format!("{}_upper", profile_name));

            let commit_filter = CommitFilterConfig {
                force_files: force_files.into_iter().map(PathBuf::from).collect(),
                skip_dirs: skip_dirs.into_iter().map(PathBuf::from).collect(),
                skip_files: skip_files.into_iter().collect(),
                skip_zero_permissions: skip_zero_perms,
                skip_regex_patterns: skip_regex,
                skip_glob_patterns: skip_glob,
                skip_empty_files_in_dirs: skip_empty_in.into_iter().map(PathBuf::from).collect(),
                skip_homedir,
                skip_rootdir,
                use_rootfs_preset: !no_rootfs_preset,
            };

            let config = SandBoxConfig {
                rootfs: rootfs.clone(),
                profile: profile.clone(),
                overlay: OverlayConfig {
                    inode_mode: settings_overlay_inode_mode(),
                    action: settings_overlay_action(),
                    ..Default::default()
                },
                commit_filter,
                ..Default::default()
            };

            if do_ask {
                AskConfig::new(&upper, &profiled_rootfs, &config)
                    .actions([
                        UpperChoice::Commit,
                        UpperChoice::CommitAtomic,
                        UpperChoice::Discard,
                        UpperChoice::Preserve,
                        UpperChoice::Cancel,
                    ])
                    .ask()?;
            } else {
                let choice = if do_commit {
                    UpperChoice::Commit
                } else if do_commit_atomic {
                    UpperChoice::CommitAtomic
                } else {
                    UpperChoice::Discard
                };

                if !no_confirm {
                    let confirmed = DialogConfig::new(format!(
                        "Apply '{}' to upper layer at '{}'?",
                        choice.label(),
                        upper.display()
                    ))
                    .options(["Yes, proceed", "Cancel"])
                    .keys(['y', 'q'])
                    .ask_or_skip(no_confirm)?;

                    if confirmed != 0 {
                        return Ok(());
                    }
                }

                apply_choice(&choice, &upper, &profiled_rootfs, &config)?;
            }

            return Ok(());
        }

        let profiles = list_available_profiles(&rootfs, None, true, use_overlay);

        print_overlay_status(
            use_overlay,
            settings_overlay_action(),
            settings_overlay_inode_mode(),
            profiles,
        )
    }
}
