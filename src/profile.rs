use crate::settings::settings_rootfs_dir;
use flexiargs::{Arg, parse_into_vars};
use sandbox_utils::{SEPARATOR, render_table, set_profile, confirm_action};
use std::collections::VecDeque;
use std::error::Error;
use std::fs;

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
        let (mut profile, mut remove, mut rename, mut no_confirm) = (None, false, None, false);

        let mut rules = [
            Arg::bool(None, "--remove", &mut remove),
            Arg::bool(None, "--no-confirm", &mut no_confirm),
            Arg::value(Some("-R"), "--rootfs", "directory", &mut rootfs),
            Arg::option(None, "--set", "name", &mut profile),
            Arg::option(None, "--rename", "new_name", &mut rename),
        ];

        parse_into_vars("profile", &mut rules, args).ok()?;
        drop(rules);

        let action_requested = profile.is_some() || remove || rename.is_some();

        if action_requested {
            if profile.is_none() {
                return Err(
                    "You must specify a profile using --set <name> to perform an action.".into(),
                );
            }
        } else {
            let mut profiles = Vec::new();

            if let Ok(entries) = fs::read_dir(&rootfs) {
                for entry in entries.flatten() {
                    let path = entry.path();

                    if path.is_dir() {
                        let dir_name = path.file_name().unwrap().to_string_lossy();
                        if !dir_name.ends_with("_upper") && dir_name.contains("_rootfs_") {
                            let parts: Vec<&str> = dir_name.split("_rootfs_").collect();
                            if parts.len() == 2 {
                                profiles.push((parts[0].to_string(), dir_name.to_string()));
                            }
                        }
                    }
                }
            }

            println!(
                "{}\nSelected rootfs dir:\n  --> {}\n",
                SEPARATOR,
                rootfs.display()
            );
            render_table(profiles);
            println!("{}", SEPARATOR);
            return Ok(());
        }

        let target_profile = set_profile(&profile.unwrap());
        let target_path = rootfs.join(&target_profile);
        let upper_path = rootfs.join(format!("{}_upper", target_profile));

        if remove {
            if target_path.exists() {
                let msg = format!("This will permanently remove profile '{}'", target_profile);
                if !confirm_action(&msg, no_confirm)? {
                    return Ok(());
                }

                fs::remove_dir_all(&target_path)?;
                if upper_path.exists() {
                    fs::remove_dir_all(&upper_path)?;
                }
                println!("Profile '{}' successfully removed.", target_profile);
            } else {
                return Err(format!("Profile '{}' not found at {}", target_profile, target_path.display()).into());
            }
        } else if let Some(new_name) = rename {
            let new_dir_name = set_profile(&new_name);
            let new_path = rootfs.join(&new_dir_name);
            let new_upper_path = rootfs.join(format!("{}_upper", new_dir_name));

            if target_path.exists() {
                fs::rename(&target_path, &new_path)?;
                if upper_path.exists() {
                    fs::rename(&upper_path, &new_upper_path)?;
                }
                println!("Profile '{}' renamed to '{}'.", target_profile, new_name);
            } else {
                return Err(format!("Profile '{}' not found.", target_profile).into());
            }
        }

        Ok(())
    }
}
