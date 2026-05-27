//! Alpine Package Manager (apk) wrapper module.
//!
//! This module provides a bridge between ALPack commands and the native
//! Alpine `apk` manager. It handles command aliasing (e.g., 'install' to 'add')
//! and ensures commands are executed within the correct rootfs context.

use crate::settings::{settings_profile, settings_rootfs_dir};
use flexiargs::{Arg, ParserOptions, parse_into_vars};
use sandbox_utils::{SandBox, SandBoxConfig, map_result};
use std::collections::VecDeque;
use std::error::Error;
use std::path::PathBuf;

/// Controller for interacting with the Alpine Package Manager.
pub struct Apk<'a> {
    /// The specific apk subcommand to run.
    cmd: &'a str,
    /// Additional arguments passed to the apk command.
    args: Vec<String>,
    /// Optional rootfs directory override.
    rootfs: Option<PathBuf>,
    /// Optional profile name override.
    profile: Option<String>,
}

impl<'a> Apk<'a> {
    /// Creates a new `Apk` instance with provided execution details.
    pub fn new(
        cmd: &'a str,
        args: Vec<String>,
        rootfs: Option<PathBuf>,
        profile: Option<String>,
    ) -> Self {
        Self {
            cmd,
            args,
            rootfs,
            profile,
        }
    }

    /// Orchestrates the execution of the Alpine Package Manager (apk).
    ///
    /// This method maps ALPack's internal commands and aliases to their
    /// corresponding `apk` operations. It ensures that any command passed
    /// is properly routed or returns a helpful error if none is specified.
    ///
    /// # Returns
    /// - `Ok(())` if the command is successfully dispatched.
    /// - `Err` if no command is provided or if execution fails.
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        if self.cmd == "apk" {
            return self.run_apk("apk");
        }

        let cmd_deque: VecDeque<String> = self.cmd.split_whitespace().map(String::from).collect();

        let mut rules = [
            Arg::action(Some("-i"), "add|install", || self.run_apk("apk add")),
            Arg::action(Some("-r"), "del|remove", || self.run_apk("apk del")),
            Arg::action(Some("-s"), "search", || self.run_apk("apk search")),
            Arg::action(None, "fix", || self.run_apk("apk fix")),
            Arg::action(Some("-u"), "update", || {
                self.run_apk("apk update && apk upgrade")
            }),
        ];

        let opts = ParserOptions {
            ignore_help: true,
            ..Default::default()
        };

        parse_into_vars(&mut rules, cmd_deque, opts).ok()
    }

    /// Executes an `apk` command inside the root filesystem environment.
    ///
    /// # Parameters
    /// - `cmd`: The base `apk` command to execute (e.g., "add", "del", "update").
    ///
    /// # Returns
    /// - `Ok(())` on success.
    /// - `Err(Box<dyn Error>)` if execution fails.
    fn run_apk(&self, cmd: &str) -> Result<(), Box<dyn Error>> {
        let rootfs = match &self.rootfs {
            Some(path) => path.clone(),
            None => settings_rootfs_dir(),
        };

        let profile = match &self.profile {
            Some(name) => name.clone(),
            None => settings_profile(),
        };

        let run_cmd = if self.args.is_empty() {
            cmd.to_string()
        } else {
            format!("{} {}", cmd, self.args.join(" "))
        };

        let config = SandBoxConfig {
            rootfs,
            run_cmd,
            profile,
            use_root: true,
            secure_rootfs: true,
            ..Default::default()
        };

        map_result(SandBox::run(config))?;
        Ok(())
    }
}
