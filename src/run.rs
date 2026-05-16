//! Execution handler for isolated processes.
//!
//! This module parses flags for the `run` subcommand, allowing users to
//! override the rootfs path, inject custom bind mounts, and define the
//! command to be executed within the sandbox.

use crate::settings::{
    settings_overlay_action, settings_overlay_inode_mode, settings_rootfs_dir, settings_use_overlay,
};
use flexiargs::{Arg, parse_into_vars};
use sandbox_utils::{OverlayAction, OverlayConfig, SandBox, SandBoxConfig, map_result};
use std::collections::VecDeque;
use std::error::Error;

/// Manager for the `run` subcommand execution.
pub struct Run {
    /// Arguments captured after the `run` keyword.
    remaining_args: Vec<String>,
}

impl Run {
    /// Creates a new `Run` instance with the provided arguments.
    pub fn new(remaining_args: Vec<String>) -> Self {
        Run { remaining_args }
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
        let mut rootfs = settings_rootfs_dir();
        let args: VecDeque<String> = self.remaining_args.iter().cloned().collect();

        let (mut cmd_args, mut remain_args) = (Vec::new(), Vec::new());
        let mut args_bind = String::new();
        let (mut use_root, mut ignore_extra_bind, mut secure_rootfs) = (false, false, false);
        let mut use_overlay = settings_use_overlay();
        let mut overlay_action = settings_overlay_action();
        let inode_mode = settings_overlay_inode_mode();

        let mut rules = [
            Arg::bool(Some("-0"), "--root", &mut use_root),
            Arg::bool(Some("-i"), "--ignore-extra-binds", &mut ignore_extra_bind),
            Arg::bool(Some("-s"), "--secure-rootfs", &mut secure_rootfs),
            Arg::value(Some("-b"), "--bind", "directory", &mut args_bind),
            Arg::collect_list(Some("-c"), "--command", "directory", &mut cmd_args),
            Arg::value(Some("-R"), "--rootfs", "directory", &mut rootfs),
            Arg::action(Some("-e"), "--ephemeral", || {
                use_overlay = true;
                overlay_action = OverlayAction::Discard;
            }),
        ];

        parse_into_vars("run", &mut rules, args).collect_rest(&mut remain_args)?;
        drop(rules);

        cmd_args.extend(remain_args);

        let run_cmd = if cmd_args.is_empty() {
            String::new()
        } else {
            cmd_args.join(" ")
        };

        let overlay = OverlayConfig {
            use_overlay,
            inode_mode,
            action: overlay_action,
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
            ..Default::default()
        };

        map_result(SandBox::run(config))?;
        Ok(())
    }
}
