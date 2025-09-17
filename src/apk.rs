use crate::command::Command;
use crate::settings::Settings;
use std::error::Error;

pub struct Apk {
    name: String,
    command: Option<String>,
    remaining_args: Vec<String>,
    rootfs: Option<String>
}

impl Apk {
    pub fn new(name: String, command: Option<String>, remaining_args: Vec<String>, rootfs: Option<String>) -> Self {
        Apk {
            name,
            command,
            remaining_args,
            rootfs
        }
    }

    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        match &self.command.as_deref() {
            Some("add") | Some("install") => {
                self.run_apk("apk add")?;
                Ok(())
            },
            Some("del") | Some("remove") => {
                self.run_apk("apk del")?;
                Ok(())
            },
            Some("-u") | Some("update") => {
                self.run_apk("apk update; apk upgrade")?;
                Ok(())
            },
            Some("-s") | Some("search") => {
                self.run_apk("apk search")?;
                Ok(())
            },
            Some("fix") => {
                self.run_apk("apk fix")?;
                Ok(())
            }
            Some(other) => {
                self.run_apk(format!("apk {}", other).as_str())?;
                Ok(())
            },
            None => {
                Err(format!("{c}: apk: no command specified\nUse '{c} --help' to see available options.", c = self.name.clone()).into())
            }
        }
    }

    /// Executes an `apk` command inside the root filesystem environment.
    ///
    /// # Parameters
    /// - `cmd`: The base `apk` command to execute (e.g., "add", "del", "update").
    ///
    /// # Returns
    /// - `Ok(())` on success.
    /// - `Err(Box<dyn Error>)` if execution fails.
    ///
    /// # Example
    /// ```
    /// self.run_apk("add")?;
    /// ```
    fn run_apk(&self, cmd: &str) -> Result<(), Box<dyn Error>> {
        let get_rootfs = match self.rootfs.clone().unwrap_or_default().is_empty() {
            false => self.rootfs.clone().unwrap(),
            true => {
                let sett = Settings::load_or_create();
                sett.set_rootfs()
            }
        };

        Command::run(get_rootfs, None,
                     Some(format!("{cmd} {}", self.remaining_args.join(" "))),
                     true, true, false)?;
        Ok(())
    }
}
