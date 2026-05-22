use crate::settings::{Settings, settings_profile, settings_rootfs_dir};
use flexiargs::{Arg, parse_into_vars};
use sandbox_utils::{
    handle_removal, handle_rename, list_available_profiles, print_profile_table, set_profile,
};
use std::collections::VecDeque;
use std::error::Error;

/// Manager for the `overlay` subcommand execution.
pub struct Profile {
    /// Arguments captured after the `overlay` keyword.
    args: Vec<String>,
}

impl Profile {
    /// Creates a new `Overlay` instance with the provided arguments.
    pub fn new(args: Vec<String>) -> Self {
        Self { args }
    }

    /// Parses profile arguments to list, remove, or rename sandbox rootfs directories.
    ///
    /// # Returns
    /// * `Ok(())` - If the operation (list, remove, or rename) completes successfully.
    /// * `Err` - If an invalid argument is passed, required parameters are missing or filesystem operations fail.
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let args: VecDeque<String> = self.args.iter().cloned().collect();
        let mut rootfs = settings_rootfs_dir();
        let (mut profile, mut rename) = (None, None);
        let (mut remove, mut no_confirm, mut set_default) = (false, false, false);

        let mut rules = [
            Arg::bool(None, "--remove", &mut remove),
            Arg::bool(None, "--no-confirm", &mut no_confirm),
            Arg::bool(None, "--default", &mut set_default),
            Arg::value(Some("-R"), "--rootfs", "directory", &mut rootfs),
            Arg::option(None, "--set", "name", &mut profile),
            Arg::option(None, "--rename", "new_name", &mut rename),
        ];

        parse_into_vars("profile", &mut rules, args).strict().ok()?;
        drop(rules);

        let action_requested = profile.is_some() || remove || rename.is_some() || set_default;

        if !action_requested {
            let profiles = list_available_profiles(&rootfs, Some(&settings_profile()), false);
            return print_profile_table(&rootfs, profiles);
        }

        if profile.is_none() {
            return Err(
                "You must specify a profile using --set <name> to perform an action.".into(),
            );
        }

        let str_profile = profile.unwrap_or_default();
        let prof_path = set_profile(&str_profile);
        let target = rootfs.join(&prof_path);
        let upper_path = rootfs.join(format!("{}_upper", prof_path));

        handle_removal(remove, &target, &upper_path, &prof_path, no_confirm)?;
        handle_rename(rename.clone(), &rootfs, &target, &upper_path, &prof_path)?;

        if set_default || str_profile == settings_profile() {
            let target_name = rename.as_deref().unwrap_or(&str_profile);
            Settings::load().set_default_profile(&target_name)?;
        }

        Ok(())
    }
}
