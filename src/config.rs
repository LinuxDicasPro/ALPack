//! Global configuration management for ALPack.
//!
//! This module handles the `config` subcommand, allowing users to modify
//! persistent settings such as rootfs isolation tools, release channels,
//! and directory paths via CLI arguments.

use crate::help::HELP_RULES;
use crate::settings::{Settings, settings_cache_dir, settings_rootfs_dir};
use flexiargs::{Arg, parse_into_vars};
use sandbox_utils::{InodeMode, OverlayAction, config_dir, confirm_action};
use std::collections::VecDeque;
use std::error::Error;

/// Configuration manager for updating application settings.
pub struct Config {
    /// List of command-line arguments to be parsed.
    args: Vec<String>,
}

impl Config {
    /// Creates a new `Config` instance with a vector of string arguments passed to the config.
    pub fn new(args: Vec<String>) -> Self {
        Self { args }
    }

    /// Parses arguments and updates the persistent settings.
    ///
    /// Processes flags for isolation tools (`--use-proot`, `--use-bwrap`),
    /// release channels, and directory configurations. Changes are displayed
    /// to the user and saved to the configuration file if modifications occur.
    ///
    /// # Returns
    /// * `Ok(())` - If configuration was successfully updated and saved.
    /// * `Err` - If an invalid argument is provided or parsing fails.
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let args: VecDeque<String> = self.args.iter().cloned().collect();
        let sett = Settings::load();
        let (mut clean_cache, mut reset_config, mut purge, mut no_confirm) =
            (false, false, false, false);

        let mut rules = [
            Arg::bool(None, "--clean-cache", &mut clean_cache),
            Arg::bool(None, "--reset-config", &mut reset_config),
            Arg::bool(None, "--purge", &mut purge),
            Arg::bool(None, "--no-confirm", &mut no_confirm),
            Arg::rw_bool(None, "--enable-overlay|--use-overlay", &sett.use_overlay),
            Arg::rw_set(None, "--disable-overlay", false, &sett.use_overlay),
            Arg::rw_set(None, "--use-persistent-inode", InodeMode::Persistent, &sett.overlay_inode_mode),
            Arg::rw_set(None, "--use-virtual-inode", InodeMode::Virtual, &sett.overlay_inode_mode),
            Arg::rw_set(None, "--overlay-action-discard", OverlayAction::Discard, &sett.overlay_action),
            Arg::rw_set(None, "--overlay-action-commit", OverlayAction::Commit, &sett.overlay_action),
            Arg::rw_set(None, "--overlay-action-commit-atomic", OverlayAction::CommitAtomic, &sett.overlay_action),
            Arg::rw_set(None, "--overlay-action-preserve", OverlayAction::Preserve, &sett.overlay_action),
            Arg::rw_set(None, "--use-proot", "proot".to_string(), &sett.cmd_rootfs),
            Arg::rw_set(None, "--use-bwrap", "bwrap".to_string(), &sett.cmd_rootfs),
            Arg::rw_set(None, "--mirror-stable", "latest-stable".to_string(), &sett.release),
            Arg::rw_set(None, "--mirror-edge", "edge".to_string(), &sett.release),
            Arg::rw_value(None, "--cache-dir", "directory", &sett.cache_dir),
            Arg::rw_value(None, "--rootfs-dir", "directory", &sett.rootfs_dir),
            Arg::rw_value(None, "--output-dir", "directory", &sett.output_dir),
            Arg::rw_value(None, "--default-mirror", "mirror", &sett.default_mirror),
        ];

        parse_into_vars("aports", &mut rules, HELP_RULES, args).strict().ok()?;
        drop(rules);

        if clean_cache || reset_config || purge {
            if clean_cache {
                if confirm_action("This will clear all cached files", no_confirm)? {
                    obliterate::ensure_removed(settings_cache_dir())?;
                    println!("Cache cleared.");
                }
            }

            if reset_config {
                if confirm_action("This will reset your configuration to defaults", no_confirm)? {
                    Settings::default().save()?;
                    println!("Configuration reset to default.");
                }
            }

            if purge {
                if confirm_action(
                    "This will PURGE all ALPack data (rootfs, cache, and config)",
                    no_confirm,
                )? {
                    let paths = [settings_cache_dir(), settings_rootfs_dir(), config_dir()];
                    for path in paths {
                        if path.exists() {
                            obliterate::ensure_removed(path)?;
                        }
                    }
                    println!("All ALPack data purged.");
                }
            }
            return Ok(());
        }

        sett.show_config_changes();
        if !self.args.is_empty() {
            sett.save()?;
        }
        Ok(())
    }
}
