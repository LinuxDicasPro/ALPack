use crate::parse_key_value;
use crate::settings::Settings;
use crate::utils::_parse_key_value;

use std::collections::VecDeque;
use std::error::Error;

pub struct Config {
    name: String,
    remaining_args: Vec<String>
}

impl Config {
    pub fn new(name: String, remaining_args: Vec<String>) -> Self {
        Config {
            name,
            remaining_args
        }
    }

    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        let mut args: VecDeque<_> = self.remaining_args.clone().into();
        let mut sett = Settings::load_or_create();

        while let Some(arg) = args.pop_front() {
            match arg.as_str() {
                "--use-proot" => {
                    sett.cmd_rootfs = "proot".to_string();
                },
                "--use-bwrap" => {
                    sett.cmd_rootfs = "bwrap".to_string();
                },
                "--use-latest-stable" => {
                    sett.release = "latest-stable".to_string();
                },
                "--use-edge" => {
                    sett.release = "edge".to_string();
                },
                a if a.starts_with("--cache-dir=") => {
                    sett.cache_dir = parse_key_value!("config", "directory", arg)?.unwrap();
                }
                "--cache-dir" => {
                    sett.cache_dir = parse_key_value!("config", "directory", arg, args.pop_front().unwrap_or_default())?.unwrap();
                },
                a if a.starts_with("--rootfs-dir=") => {
                    sett.rootfs_dir = parse_key_value!("config", "directory", arg)?.unwrap();
                }
                "--rootfs-dir" => {
                    sett.rootfs_dir = parse_key_value!("config", "directory", arg, args.pop_front().unwrap_or_default())?.unwrap();
                },
                a if a.starts_with("--output-dir=") => {
                    sett.rootfs_dir = parse_key_value!("config", "directory", arg)?.unwrap();
                }
                "--output-dir" => {
                    sett.rootfs_dir = parse_key_value!("config", "directory", arg, args.pop_front().unwrap_or_default())?.unwrap();
                },
                a if a.starts_with("--default-mirror=") => {
                    sett.default_mirror = parse_key_value!("config", "mirror", arg)?.unwrap();
                }
                "--default-mirror" => {
                    sett.default_mirror = parse_key_value!("config", "mirror", arg, args.pop_front().unwrap_or_default())?.unwrap();
                },
                _ => {
                    return Err(format!("{c}: aports: invalid argument '{arg}'\nUse '{c} --help' to see available options.", c = self.name).into())
                }
            }
        }

        sett.show_config_changes();
        if !self.remaining_args.is_empty() {
            sett.save()?;
        }
        Ok(())
    }
}
