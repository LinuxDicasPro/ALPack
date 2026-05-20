//! Alpine Package Manager (apk) wrapper module.
//!
//! This module provides a bridge between ALPack commands and the native
//! Alpine `apk` manager. It handles command aliasing (e.g., 'install' to 'add')
//! and ensures commands are executed within the correct rootfs context.

use crate::settings::{settings_profile, settings_rootfs_dir};
use flexiargs::{Arg, parse_into_vars};
use sandbox_utils::{SandBox, SandBoxConfig, map_result};
use std::error::Error;
use std::path::PathBuf;

/// Controller for interacting with the Alpine Package Manager.
pub struct Apk {
    /// The specific apk subcommand to run.
    cmd: Option<String>,
    /// Additional arguments passed to the apk command.
    args: Vec<String>,
    /// Optional rootfs directory override.
    rootfs: Option<PathBuf>,
    /// Optional profile name override.
    profile: Option<String>,
}

impl Apk {
    /// Creates a new `Apk` instance with provided execution details.
    pub fn new(
        cmd: Option<String>,
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
        let mut remain_args = Vec::new();
        let cmd_deque = self
            .cmd
            .as_deref()
            .map(|s| s.split_whitespace().map(String::from).collect())
            .unwrap_or_default();

        let mut rules = [
            Arg::action(Some("add"), "install", || self.run_apk("apk add")),
            Arg::action(Some("del"), "remove", || self.run_apk("apk del")),
            Arg::action(Some("-s"), "search", || self.run_apk("apk search")),
            Arg::action(None, "fix", || self.run_apk("apk fix")),
            Arg::action(Some("-u"), "update", || {
                self.run_apk("apk update && apk upgrade")
            }),
        ];

        parse_into_vars("apk", &mut rules, cmd_deque)
            .passthrough()
            .require_args()?
            .collect_rest(&mut remain_args)?;

        remain_args
            .first()
            .map_or(Ok(()), |other| self.run_apk(&format!("apk {other}")))
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
