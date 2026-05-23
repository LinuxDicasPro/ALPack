use crate::settings::{
    settings_overlay_action, settings_overlay_inode_mode, settings_rootfs_dir, settings_use_overlay,
};
use flexiargs::{Arg, parse_into_vars};
use sandbox_utils::{list_available_profiles, print_overlay_status};
use std::collections::VecDeque;
use std::error::Error;
use crate::help::HELP_RULES;

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

    /// todo
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let args: VecDeque<String> = self.args.iter().cloned().collect();
        let mut rootfs = settings_rootfs_dir();
        let use_overlay = settings_use_overlay();

        let mut rules = [Arg::value(Some("-R"), "--rootfs", "directory", &mut rootfs)];

        parse_into_vars("overlay", &mut rules, HELP_RULES, args).strict().ok()?;
        drop(rules);

        let profiles = list_available_profiles(&rootfs, None, true, use_overlay);

        print_overlay_status(
            use_overlay,
            settings_overlay_action(),
            settings_overlay_inode_mode(),
            profiles,
        )
    }
}
