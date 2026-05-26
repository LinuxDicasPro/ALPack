//! Execution handler for isolated processes.
//!
//! This module parses flags for the `run` subcommand, allowing users to
//! override the rootfs path, inject custom bind mounts, and define the
//! command to be executed within the sandbox.

use crate::help::HELP_RULES;
use crate::settings::{
    settings_overlay_action, settings_overlay_inode_mode, settings_profile, settings_rootfs_dir,
    settings_use_overlay,
};
use flexiargs::{Arg, parse_into_vars, ParserOptions};
use sandbox_utils::{OverlayAction, OverlayConfig, SandBox, SandBoxConfig, map_result};
use std::collections::VecDeque;
use std::error::Error;

/// Manager for the `run` subcommand execution.
pub struct Run {
    /// Arguments captured after the `run` keyword.
    args: Vec<String>,
}

impl Run {
    /// Creates a new `Run` instance with the provided arguments.
    pub fn new(args: Vec<String>) -> Self {
        Self { args }
    }

    /// Orchestrates the parsing of arguments and triggers the command execution.
    ///
    /// It handles specific flags like `--root`, `--bind-args`, and `--command`.
    /// If no command is provided, it defaults to the shell defined in the `Command` module.
    ///
    /// # Returns
    /// * `Ok(())` - If the command was executed successfully.
    /// * `Err` - If an invalid argument is found or the execution fails.
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let args: VecDeque<String> = self.args.iter().cloned().collect();
        let mut rootfs = settings_rootfs_dir();
        let (mut cmd_args, mut remain_args) = (Vec::new(), Vec::new());
        let (mut args_bind, mut profile) = (String::new(), settings_profile());
        let (mut use_root, mut ignore_extra_bind, mut secure_rootfs) = (false, false, false);
        let (mut use_overlay, mut is_overlay, mut is_ephemeral) =
            (settings_use_overlay(), false, false);
        let mut action = settings_overlay_action();
        let inode_mode = settings_overlay_inode_mode();

        let mut rules = [
            Arg::bool(Some("-0"), "--root", &mut use_root),
            Arg::bool(Some("-e"), "--ephemeral", &mut is_ephemeral),
            Arg::bool(Some("-o"), "--overlay", &mut is_overlay),
            Arg::bool(Some("-i"), "--ignore-extra-binds", &mut ignore_extra_bind),
            Arg::bool(Some("-s"), "--secure-rootfs", &mut secure_rootfs),
            Arg::value(Some("-b"), "--bind", "directory", &mut args_bind),
            Arg::value(Some("-R"), "--rootfs", "directory", &mut rootfs),
            Arg::value(Some("-p"), "--profile", "profile", &mut profile),
            Arg::collect_list(Some("-c"), "--command", "directory", &mut cmd_args),
        ];

        let opts = ParserOptions {
            subcommand: "run",
            help_rules: HELP_RULES,
            strict: false,
            ..Default::default()
        };

        if parse_into_vars(&mut rules, args, opts)
            .collect_rest(&mut remain_args)?
            .help_requested()
        {
            return Ok(());
        }

        drop(rules);

        cmd_args.extend(remain_args);

        if is_ephemeral {
            use_overlay = true;
            action = OverlayAction::Discard;
        } else if is_overlay {
            use_overlay = true;
            action = OverlayAction::Preserve;
        }

        let run_cmd = if cmd_args.is_empty() {
            String::new()
        } else {
            cmd_args.join(" ")
        };

        let overlay = OverlayConfig {
            use_overlay,
            inode_mode,
            action,
            ..Default::default()
        };

        let config = SandBoxConfig {
            rootfs,
            run_cmd,
            args_bind,
            use_root,
            ignore_extra_bind,
            secure_rootfs,
            overlay,
            profile,
            ..Default::default()
        };

        map_result(SandBox::run(config))?;
        Ok(())
    }
}
