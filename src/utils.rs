use std::{env, fs, io};
use std::error::Error;
use std::ops::Add;
use std::path::Path;
use walkdir::WalkDir;

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
pub(crate) fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    println!("copy {} to {}", src.display(), dst.display());
    let dir_name = src.file_name().ok_or_else(|| { io::Error::new(io::ErrorKind::Other, "invalid directory") })?;
    let dest_root = dst.join(dir_name);

    for entry in WalkDir::new(src) {
        let entry = entry?;
        let relative_path = entry.path().strip_prefix(src).unwrap();
        let dest_path = dest_root.join(relative_path);

        if entry.file_type().is_dir() {
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
