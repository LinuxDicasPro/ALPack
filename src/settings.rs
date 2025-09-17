use serde::{Deserialize, Serialize};
use std::{env, fs, io, path::PathBuf};
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    #[serde(default = "default_mirror")]
    pub default_mirror: String,

    #[serde(default = "default_cache")]
    pub cache_dir: String,

    #[serde(default = "default_rootfs")]
    pub rootfs_dir: String,

    #[serde(default = "default_cmd_rootfs")]
    pub cmd_rootfs: String,

    #[serde(default = "default_release")]
    pub release: String,

    #[serde(default = "default_output")]
    pub output_dir: String
}

fn default_mirror() -> String { "https://dl-cdn.alpinelinux.org/alpine/".to_string() }
fn default_cache() -> String { format!("{}/.cache/ALPack", env!("HOME")) }
fn default_rootfs() -> String { format!("{}/.ALPack", env!("HOME")) }
fn default_cmd_rootfs() -> String { "proot".to_string() }
fn default_release() -> String { "latest-stable".to_string() }
fn default_output() -> String { String::new() }

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_mirror: default_mirror(),
            cache_dir: default_cache(),
            rootfs_dir: default_rootfs(),
            cmd_rootfs: default_cmd_rootfs(),
            release: default_release(),
            output_dir: default_output()
        }
    }
}

impl Settings {

    /// Loads the configuration from the config file, or creates a default one if it doesn't exist or is invalid.
    ///
    /// # Examples
    /// ```
    /// let settings = Settings::load_or_create();
    /// ```
    pub fn load_or_create() -> Self {
        let path = PathBuf::from(format!("{}/.config/ALPack/config.toml", env!("HOME")));

        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    if content.is_empty() {
                        eprintln!("\x1b[1;33mWarning\x1b[0m: config file is empty. Using default settings.");
                        Settings::create(path)
                    } else {
                        toml::from_str(&content).unwrap_or_else(|_| {
                            eprintln!("\x1b[1;33mWarning\x1b[0m: Failed to parse config file. Using default settings.");
                            Settings::create(path)
                        })
                    }
                },
                Err(e) => {
                    eprintln!("\x1b[1;33mWarning\x1b[0m: Failed to get metadata for config file: {e}");
                    Settings::create(path)
                }
            }
        } else {
            eprintln!("\x1b[1;33mWarning\x1b[0m: Config file not found, creating a new one...");
            Settings::create(path)
        }
    }

    /// Creates a new configuration file with default values.
    ///
    /// # Parameters
    /// - `path`: Path where the configuration file should be created.
    ///
    /// # Returns
    /// - A `Settings` struct containing default values.
    ///
    /// # Examples
    /// ```
    /// let settings = Settings::create(PathBuf::from("/some/path"));
    /// ```
    fn create(path: PathBuf) -> Self {
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        let default = Settings::default();

        if let Err(e) = fs::write(&path, toml::to_string_pretty(&default).unwrap()) {
            eprintln!("\x1b[1;33mWarning\x1b[0m: Failed to write default config file: {e}");
        }

        default
    }

    /// Saves the current configuration to the default config file path.
    ///
    /// # Returns
    /// - `Ok(())` if the file was successfully written.
    /// - `Err` if the config could not be serialized or written.
    ///
    /// # Examples
    /// ```
    /// let settings = Settings::load_or_create();
    /// settings.save().unwrap();
    /// ```
    pub fn save(&self) -> io::Result<()> {
        let path = PathBuf::from(format!("{}/.config/ALPack/config.toml", env!("HOME")));
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        let toml_data = toml::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        fs::write(path, toml_data)
    }

    /// Displays the current configuration from the disk and compares it with the in-memory config.
    ///
    /// # Output
    /// - Fields that differ will be shown in the format: `field_name: old_value -> new_value`.
    /// - Fields that are new or unchanged will be shown as: `field_name: value`.
    ///
    /// # Example
    /// ```
    /// let settings = Settings::load_or_create();
    /// settings.show_config_changes();
    /// ```
    pub fn show_config_changes(&self) {
        let path = format!("{}/.config/ALPack/config.toml", env!("HOME"));
        let _current_disk_config = fs::read_to_string(&path).ok().and_then(|s| toml::from_str::<Settings>(&s).ok());
        let mut rows: Vec<(String, String)> = Vec::new();

        macro_rules! show_field {
            ($field:ident) => {
                let field_name = stringify!($field).to_string();
                let mut new_val = self.$field.to_string();
                let value_str = if let Some(old) = &_current_disk_config {
                    let mut old_val = old.$field.to_string();
                    if field_name == "output_dir" {
                        let val = "Current Directory or Home FallBack".to_string();
                        if new_val == String::new() {
                            new_val = val.clone();
                        }
                        if old_val == String::new() {
                            old_val = val.clone();
                        }
                    }
                    if old_val != new_val {
                        format!("\x1b[1;31m{old_val}\x1b[0m -> \x1b[1;32m{new_val}\x1b[0m")
                    } else {
                        new_val.clone()
                    }
                } else {
                    new_val.clone()
                };
                rows.push((field_name, value_str));
            };
        }

        show_field!(default_mirror);
        show_field!(cache_dir);
        show_field!(rootfs_dir);
        show_field!(cmd_rootfs);
        show_field!(release);
        show_field!(output_dir);

        let key_width = rows.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
        let val_width = rows.iter().map(|(_, v)| v.len()).max().unwrap_or(0);

        let output = &mut format!("╔═{}═══╦═{}═══╗\n", "═".repeat(key_width), "═".repeat(val_width));

        for (k, v) in rows {
            if v.find('>').is_some() {
                output.push_str(&format!("║ {:<key_width$}   ║ {:<x$}   ║\n", k, v, x = val_width + 22),);
            } else {
                output.push_str(&format!("║ {:<key_width$}   ║ {:<val_width$}   ║\n", k, v));
            }
        }

        output.push_str(&format!("╚═{}═══╩═{}═══╝\n", "═".repeat(key_width), "═".repeat(val_width)));

        print!("{output}");
    }

    /// Determines the output directory for the application.
    ///
    /// # Returns
    /// * `Ok(String)` - The path of the output directory as a string.
    /// * `Err` - An `io::Error` if there was a problem accessing the current directory.
    ///
    /// # Examples
    /// ```
    /// let out_dir = set_output_dir()?;
    /// println!("Output directory: {}", out_dir);
    /// ```
    pub fn set_output_dir() -> io::Result<String> {
        let current =  env::current_dir()?.display().to_string();
        let target_path = Path::new(current.as_str());

        match fs::read_dir(target_path) {
            Ok(_) => return Ok(target_path.display().to_string()),
            Err(ref e) if e.kind() == io::ErrorKind::PermissionDenied => {
                eprintln!("\x1b[1;33mWarning\x1b[0m: Permission denied to create '{}', using default directory instead...", target_path.display());
            }
            Err(e) => {
                return Err(e);
            }
        }

        let fallback_path = Path::new( env!("HOME"));
        Ok(fallback_path.display().to_string())
    }

    /// Determines the root filesystem directory for the application.
    ///
    /// # Returns
    /// * `String` - The path to the root filesystem directory.
    ///
    /// # Examples
    /// ```
    /// let rootfs = settings.set_rootfs();
    /// println!("Rootfs directory: {}", rootfs);
    /// ```
    pub fn set_rootfs(&self) -> String {
        env::var("ALPACK_ROOTFS").unwrap_or_else(|_| self.rootfs_dir.clone())
    }

    /// Determines the cache directory for the application.
    ///
    /// # Returns
    /// * `String` - The path to the cache directory.
    ///
    /// # Examples
    /// ```
    /// let cache = settings.set_cache_dir();
    /// println!("Cache directory: {}", cache);
    /// ```
    /// 
    pub fn set_cache_dir(&self) -> String {
        env::var("ALPACK_CACHE").unwrap_or_else(|_| self.rootfs_dir.clone())
    }   
}