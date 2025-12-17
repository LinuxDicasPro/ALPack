use crate::settings::Settings;
use indicatif::{ProgressBar, ProgressStyle};
use std::error::Error;
use std::fs::File;
use std::ops::Add;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::{env, fs, io};
use walkdir_minimal::WalkDir;
use which::which;

pub const DOWNLOAD_TEMPLATE: &str =
    "{msg} {spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})";

#[macro_export]
macro_rules! parse_key_value {
    ($sub:expr, $val:expr, $arg:expr) => {
        _parse_key_value($sub, $val, $arg, None)
    };
    ($sub:expr, $val:expr, $arg:expr, $next:expr) => {
        _parse_key_value($sub, $val, $arg, Some($next))
    };
}

/// Parses a `--key=value` or `--key value` style argument.
///
/// # Parameters
/// - `sub`: The subcommand name (used in error messages).
/// - `val`: A human-readable name of the expected value (used in error messages).
/// - `key`: The key to match (e.g., "cache", "mirror").
/// - `arg`: The current argument to check (maybe `--key=value` or just `--key`).
/// - `next_args`: The next argument, if available (used if `arg` is just `--key`).
///
/// # Returns
/// - `Ok(Some(value))` if a value is successfully parsed.
/// - `Err` if the argument is missing a required value.
///
/// # Example
/// ```
/// let val = _parse_key_value("install", "PATH", "cache", "--cache=/tmp".to_string(), None)?;
/// assert_eq!(val, Some("/tmp".to_string()));
/// ```
pub fn _parse_key_value(sub: &str, val: &str, arg: String, next_args: Option<String>) -> Result<Option<String>, Box<dyn Error>> {
    let prefix = arg.clone().split("=").collect::<Vec<&str>>()[0].to_string().add("=");
    let cmd = env::current_exe().unwrap().file_name().unwrap().to_str().unwrap().to_string();
    let mut value = arg.strip_prefix(&prefix).unwrap_or_default().to_string();
    let mut sp = "".to_string();

    if next_args.is_some() {
        value = next_args.unwrap_or_default();
        sp = " ".to_string();
    }

    if value.is_empty() {
        return Err(format!("{cmd}: {sub}: {arg} requires a <{val}> as argument.\nUsage: {cmd} {sub} {arg}{sp}<{val}>").into());
    }
    Ok(Some(value.to_string()))
}

/// Determines the architecture string to use.
///
/// # Returns
/// * `String` - A string representing the architecture (e.g., "x86_64", "aarch64").
///
/// # Example
/// ```
/// let arch = get_arch();
/// println!("Detected architecture: {}", arch);
/// ```
pub fn get_arch() -> String {
    env::var("ALPACK_ARCH").unwrap_or_else(
        |_| env::var("ARCH").unwrap_or_else(
            |_| env::consts::ARCH.to_string()
        )
    )
}

/// Displays a final setup message with styled formatting.
///
/// # Arguments
/// * `cmd` - The base command to be used in the suggestion (e.g., "ALPack").
///
/// # Example
/// ```
/// finish_msg_setup("ALPack".to_string());
/// ```
pub fn finish_msg_setup(cmd: String) {
    println!(
"{s}\n  Installation completed successfully!\n
  To start the environment, run:\n{b}\n{s}",
b = get_cmd_box(format!("$ {} run", cmd), Some(2), None).unwrap(), s = separator_line());
}

/// Verifies that the specified rootfs directory exists.
///
/// # Arguments
/// * `path` - A string slice that holds the path to the rootfs directory.
///
/// # Returns
/// * `Ok(())` if the directory exists.
/// * `Err` with a descriptive message if the directory does not exist.
///
/// # Example
/// ```
/// check_rootfs_exists("/path/to/rootfs")?;
/// ```
pub fn check_rootfs_exists(cmd: String, path: String) -> Result<(), Box<dyn Error>> {
    let dir = Path::new(path.as_str());
    if ! dir.is_dir() {
        return Err(format!(
"{s}\n  Error: rootfs directory not found.\n
  Expected location:
    -> {path}\n
  Please run the following command to set it up:\n{b}\n{s}",
b = get_cmd_box(format!("$ {} setup", cmd), Some(2), None)?, s = separator_line()).into())
    }
    Ok(())
}

/// Generates a stylized box containing a command string.
///
/// # Arguments
/// * `name` - The name of the command (e.g., "ALPack").
///
/// # Returns
/// A `String` that represents a multi-line box with the command inside.
///
/// # Example
/// ```
/// let box_str = get_cmd_box("$ ALPack setup");
/// println!("{}", box_str);
/// ```
pub fn get_cmd_box(name: String, repeat: Option<usize>, size: Option<usize>) -> Result<String, Box<dyn Error>> {
    let command = name;
    let width: usize = size.unwrap_or_else(|| 50);
    let rep: usize = repeat.unwrap_or_else(|| 0);

    let top =   "╔".to_string() + &"═".repeat(width - 2) + "╗";
    let bottom ="╚".to_string() + &"═".repeat(width - 2) + "╝";

    let mut middle = String::from("║ ");
    middle += command.as_str();
    middle += &" ".repeat(width - 3 - command.len());
    middle += "║";

    if rep == 0 {
        return Ok(format!("{top}\n{middle}\n{bottom}"))
    }

    Ok(format!("{r}{top}\n{r}{middle}\n{r}{bottom}", r = " ".repeat(rep)))
}

/// Generates a line composed of a repeated character.
///
/// # Returns
/// A `String` containing the repeated character line.
///
/// # Example
/// ```
/// println!("{s}", s = separator_line());
/// ```
pub fn separator_line() -> String {
    "═".repeat(60)
}

/// Recursively copies a directory and all its contents to a specified destination.
///
/// # Arguments
/// * `src` - The source directory to copy.
/// * `dst` - The destination directory where the source will be copied.
///
/// # Returns
/// * `io::Result<()>` - Ok on success, or an error if the operation fails.
///
/// # Example
/// ```
/// let src = std::path::Path::new("/home/user/test1");
/// let dst = std::path::Path::new("/home/user/output");
/// copy_dir_recursive(src, dst).expect("Failed to copy directory");
/// ```
pub fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    println!("copy {} to {}", src.display(), dst.display());
    let dir_name = src.file_name().ok_or_else(|| { io::Error::new(io::ErrorKind::Other, "invalid directory") })?;
    let dest_root = dst.join(dir_name);

    for entry in WalkDir::new(src)? {
        let entry = entry.unwrap();
        let relative_path = entry.path().strip_prefix(src).unwrap();
        let dest_path = dest_root.join(relative_path);

        if entry.file_type()?.is_dir() {
            fs::create_dir_all(&dest_path)?;
        } else {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), &dest_path)?;
        }
    }
    Ok(())
}

/// Attempts to create the target directory, falling back to a default path if permission is denied.
///
/// # Parameters
/// - `target`: The desired path to create.
///
/// # Returns
/// - `Ok(PathBuf)` with the successfully created directory path (either the target or fallback).
/// - `Err(io::Error)` if both the target and fallback directory creations fail.
///
/// # Examples
/// ```
/// let dir = create_dir_with_fallback("/opt/some_dir".to_string())?;
/// println!("Directory created or reused: {}", dir.display());
/// ```
pub fn create_dir_with_fallback(target: String) -> io::Result<PathBuf> {
    let target_path = Path::new(target.as_str());

    match fs::create_dir_all(target_path) {
        Ok(_) => return Ok(target_path.to_path_buf()),
        Err(ref e) if e.kind() == io::ErrorKind::PermissionDenied => {
            eprintln!("\x1b[1;33mWarning\x1b[0m: Permission denied to create '{}', using default directory instead...", target);
        }
        Err(e) => return Err(e)
    }

    let home = Settings::load_or_create().set_rootfs();
    let fallback_path = Path::new(&home);
    fs::create_dir_all(&fallback_path)?;
    Ok(PathBuf::from(fallback_path))
}

/// Downloads a file from the specified URL and saves it to the destination folder.
///
/// # Arguments
/// * `url` - The URL of the file to be downloaded.
/// * `dest` - The directory where the file will be saved.
/// * `filename` - The name of the file to save.
///
/// # Returns
/// * `Ok(String)` - The full path of the saved file.
/// * `Err`: An `io::Error` if the download or save fails.
///
/// # Examples
/// ```
/// let saved_path = download_file("https://url.com/file.tar.gz".to_string(),
///     "/tmp".to_string(), "file.tar.gz".to_string())?;
/// println!("File saved to: {}", saved_path);
/// ```
pub fn download_file(url: String, dest: String, filename: String) -> io::Result<String> {
    let dest_ok = create_dir_with_fallback(dest);
    let save_dest = dest_ok?.to_str().unwrap().to_string();
    let save_file = format!("{save_dest}/{filename}");

    if Path::new(&save_file).exists() {
        println!("File '{}' already exists, skipping download.", filename);
        return Ok(save_dest);
    }

    println!("Saving file to: {save_file}");
    let resp = ureq::get(url).call().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let length = resp.headers().get("Content-Length").unwrap().to_str().unwrap().parse().unwrap();

    let bar = ProgressBar::new(length);
    bar.set_message("Downloading...");
    bar.set_style(ProgressStyle::with_template(DOWNLOAD_TEMPLATE).unwrap().progress_chars("##-"));

    io::copy(&mut bar.wrap_read(resp.into_body().into_reader()), &mut File::create(save_file)?)?;
    bar.finish_with_message("Downloaded!");
    Ok(save_dest)
}

/// Returns the path to the user's local binary directory (`~/.local/bin`).
///
/// # Returns
/// * `PathBuf` - The full path to `~/.local/bin`. If the `HOME` environment
///   variable is not set, it falls back to the current directory (`.`).
fn local_bin_dir() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".local").join("bin")
}


/// Sets executable permissions on a file (Unix-only).
///
/// # Arguments
/// * `path` - Path to the file whose permissions will be modified.
///
/// # Returns
/// * `Ok(())` if permissions were successfully updated.
/// * `Err(io::Error)` if the file metadata cannot be read or permissions
///   cannot be set.
fn make_executable(path: &Path) -> io::Result<()> {
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms)
}

/// Returns the download URL for a supported rootfs command binary.
///
/// # Arguments
/// * `cmd` - The rootfs command name (e.g. `"proot"` or `"bwrap"`).
///
/// # Returns
/// * `Some(&'static str)` containing the download URL if the command
///   is supported.
/// * `None` if the command is unknown or unsupported.
fn binary_url(cmd: &str) -> Option<&'static str> {
    match cmd {
        "proot" => Some(
            "https://github.com/LinuxDicasPro/StaticHub/releases/download/proot/proot",
        ),
        "bwrap" => Some(
            "https://github.com/LinuxDicasPro/StaticHub/releases/download/bwrap/bwrap",
        ),
        _ => None,
    }
}


/// Checks whether the current system architecture is x86_64.
///
/// # Returns
/// * `true` if the architecture is `x86_64`.
/// * `false` otherwise.
fn is_x86_64() -> bool {
    env::consts::ARCH == "x86_64"
}

/// Verifies the availability of the specified rootfs command and downloads it if necessary.
///
/// Only x86_64 architecture is supported for automatic downloads. On other
/// architectures, the command must already be available in the system.
///
/// # Arguments
/// * `cmd_rootfs` - The name of the rootfs command (`"proot"` or `"bwrap"`).
///
/// # Returns
/// * `Ok(PathBuf)` - The full path to the resolved executable.
/// * `Err(io::Error)` if:
///   - The command is unsupported,
///   - The architecture is not supported,
///   - The download fails,
///   - Or file permissions cannot be set.
///
/// # Errors
/// Returns `io::ErrorKind::Unsupported` if the command is not found and
/// no binary is available for the current architecture.
pub fn verify_and_download_rootfs_command(
    cmd_rootfs: &str,
) -> io::Result<PathBuf> {
    if let Some(path) = which(cmd_rootfs).ok() {
        return Ok(path);
    }

    let local_dir = local_bin_dir();
    let local_path = local_dir.join(cmd_rootfs);

    if local_path.exists() {
        return Ok(local_path);
    }

    if !is_x86_64() {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!(
                "{} not found in the system and no binary is available for this architecture"
                cmd_rootfs
            ),
        ));
    }

    let url = binary_url(cmd_rootfs).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid cmd_rootfs",
        )
    })?;

    fs::create_dir_all(&local_dir)?;

    let downloaded = download_file(
        url.to_string(),
        local_dir.to_string_lossy().to_string(),
        cmd_rootfs.to_string(),
    )?;

    let downloaded_path = PathBuf::from(downloaded);

    make_executable(&downloaded_path)?;

    Ok(downloaded_path)
}
