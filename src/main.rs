//! ALPack - Alpine Linux RootFS Packaging Tool.
//!
//! This crate provides a comprehensive CLI for managing Alpine Linux rootfs
//! environments, allowing for automated setup, package management, and
//! repository indexing through a modular architecture.

mod apk;
mod aports;
mod aptree;
mod builder;
mod config;
mod mirror;
mod overlay;
mod profile;
mod run;
mod settings;
mod setup;
mod utils;

use crate::apk::Apk;
use crate::aports::Aports;
use crate::aptree::Aptree;
use crate::builder::Builder;
use crate::config::Config;
use crate::overlay::Overlay;
use crate::profile::Profile;
use crate::run::Run;
use crate::settings::{Settings, settings_cmd};
use crate::setup::Setup;
use flexiargs::{Arg, parse_into_vars};
use sandbox_utils::{app_name, sandbox_init, set_sandbox_tool};
use std::collections::VecDeque;
use std::env;
use std::error::Error;
use std::path::PathBuf;

/// Prints the help message and usage instructions to the console.
///
/// # Parameters
/// - `cmd`: The binary name used to invoke the program.
fn print_help(cmd: String) -> Result<(), Box<dyn Error>> {
    println!(
        "{cmd} - Alpine Linux RootFS Packaging Tool

Simple shell-based tool that allows you to create and manage Alpine Linux
rootfs containers easily using proot or bubblewrap(bwrap).

Usage:
    {cmd} <parameters> [options] [--] [ARGS...]

Available parameters:
        setup                   Initialize or configure the rootfs environment
        run                     Execute command inside the rootfs
        config                  Display or modify global configuration
        aports                  Manage local aports repository
        aptree                  Manage local Adélie Package Tree repository
        builder                 Build utility for packages and images
        apk                     Run the Alpine package manager (apk)
        add | install <ARGS>    Install packages into the rootfs
        del | remove <ARGS>     Remove packages from the rootfs
    -s, search <ARGS>           Search for available packages
    -u, update                  Update the package index and upgrade installed packages
        fix                     Attempt to fix broken packages

Options for 'setup':
        --no-cache              Disable caching during the operation
    -r, --reinstall             Reinstall packages without forcing
        --edge                  Use the edge (testing) repository
    -m, --minimal               Install only the minimal set of packages
        --mirror=<URL>          Use the specified mirror instead of the default one
        --cache=<DIR>           Specify cache directory
    -R, --rootfs=<DIR>          Specify rootfs directory
    -p, --profile=<NAME>        Specify profile name

Options for 'apk':
    -R, --rootfs=<DIR>          Specify rootfs directory
    -p, --profile=<NAME>        Specify profile name

Options for 'aports':
    -u, --update                Update the local aports repository to the latest version
    -s, --search=<PKG>          Search for a package in the Alpine aports
    -S, --strict-search=<PKG>   Search for a package with an exact name match
    -g, --get=<PKG>             Download the APKBUILD in the Alpine aports
    -R, --rootfs=<DIR>          Specify rootfs directory
    -p, --profile=<NAME>        Specify profile name

Options for 'aptree':
    -u, --update                Update the local aptree repository to the latest version
    -s, --search=<PKG>          Search for a package in the Adélie aptree
    -S, --strict-search=<PKG>   Search for a package with an exact name match
    -g, --get=<PKG>             Download the APKBUILD from the Adélie aptree
    -R, --rootfs=<DIR>          Specify rootfs directory
    -p, --profile=<NAME>        Specify profile name

Options for 'builder':
    -a, --apkbuild=<APKBUILD>   Use a specific APKBUILD file as input
        --force-key             Force regeneration of RSA signing keys
    -e, --ephemeral             Use a temporary overlay to discard changes after execution
    -R, --rootfs=<DIR>          Specify rootfs directory
    -p, --profile=<NAME>        Specify profile name

Options for 'run':
    -0, --root                  Run with root privileges inside rootfs
    -i, --ignore-extra-binds    Ignore additional bind mounts
    -s, --secure-rootfs         Minimal mounting with maximum isolation and restricted integration
    -e, --ephemeral             Use a temporary overlay to discard changes after execution
    -b, --bind-args=<ARGS>      Additional bind arguments
    -c, --command=<CMD>         Command to execute inside rootfs (can be repeated)
    -R, --rootfs=<DIR>          Specify rootfs directory
    -p, --profile=<NAME>        Specify profile name

General Options for 'config':
        --use-proot             Use 'proot' as rootfs handler (default)
        --use-bwrap             Use 'bwrap' as rootfs handler
        --use-latest-stable     Use 'latest-stable' release (default)
        --use-edge              Use 'edge' release
        --cache-dir=<DIR>       Set cache directory
        --output-dir=<DIR>      Set output directory (default current directory)
        --rootfs-dir=<DIR>      Set rootfs directory
        --profile=<NAME>        Set profile name
        --default-mirror=<URL>  Set default Alpine mirror

Overlay Options for 'config':
        --use-overlay | --enable-overlay  Enable OverlayFS to layer changes over the rootfs
        --disable-overlay                 Disable OverlayFS usage (default)
        --use-persistent-inode            Use persistent inodes for the overlay layer
        --use-virtual-inode               Use virtual inodes for the overlay layer (default)
        --overlay-action-discard          Discard all changes when the session ends
        --overlay-action-commit           Merge changes back to the rootfs after execution
        --overlay-action-commit-atomic    Merge changes to the rootfs using an atomic operation
        --overlay-action-preserve         Preserve the upper layer data without discarding it

Global Options:
    -h, --help                  Show this help message
    -V, --version               Show version

Environment variables:
    ALPACK_ARCH       Define the target architecture for rootfs (e.g., x86_64, aarch64)
    ALPACK_ROOTFS     Specify the path to the root filesystem used by ALPack
    ALPACK_CACHE      Specify the path to the cache directory used by ALPack"
    );
    Ok(())
}

/// Core logic dispatcher for the ALPack CLI.
///
/// This function handles the initial environment parsing, identifies the
/// requested command, and delegates execution to the appropriate module.
///
/// # Returns
/// - `Ok(())` if the command executes successfully.
/// - `Err` if argument parsing fails or a submodule returns an error.
fn alpack() -> Result<(), Box<dyn Error>> {
    // Todo: -w caminho, --pwd=caminho, --cwd=caminho; --kill-on-exit: limpar processos "órfãos".
    sandbox_init("ALPack", "ALPACK_ARCH")?;
    Settings::global();
    set_sandbox_tool(&settings_cmd())?;

    let mut args: VecDeque<String> = env::args().skip(1).collect();
    let cmd = args.pop_front().unwrap_or_default();
    let cmd_deque: VecDeque<String> = VecDeque::from([cmd.clone()]);

    let remain_args: Vec<String> = match cmd_deque.front().map(|s| s.as_str()) {
        Some("-h") | Some("--help") | Some("-V") | Some("--version") => Vec::new(),
        _ => args.into_iter().collect(),
    };

    let mut rules = [
        Arg::action(None, "apk", || {
            let mut args = remain_args.clone().into_iter();
            let (mut rootfs, mut subcommand, mut profile) = (None, None, None);
            let mut subargs: Vec<String> = Vec::new();

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "-R" | "--rootfs" => rootfs = args.next().map(PathBuf::from),
                    a if a.starts_with("--rootfs=") => {
                        rootfs = a.split_once('=').map(|(_, v)| PathBuf::from(v));
                    }
                    "-p" | "--profile" => profile = args.next().map(String::from),
                    a if a.starts_with("--profile=") => {
                        profile = a.split_once('=').map(|(_, v)| String::from(v));
                    }
                    _ if subcommand.is_none() => subcommand = Some(arg),
                    _ => subargs.push(arg),
                }
            }

            Apk::new(subcommand, subargs, rootfs, profile).run()
        }),
        Arg::action(
            None,
            "add|del|install|remove|search|update|fix|-s|-u",
            || Apk::new(Some(cmd.clone()), remain_args.clone(), None, None).run(),
        ),
        Arg::action(None, "aports", || Aports::new(remain_args.clone()).run()),
        Arg::action(None, "aptree", || Aptree::new(remain_args.clone()).run()),
        Arg::action(None, "builder", || Builder::new(remain_args.clone()).run()),
        Arg::action(None, "config", || Config::new(remain_args.clone()).run()),
        Arg::action(None, "overlay", || Overlay::new(remain_args.clone()).run()),
        Arg::action(None, "profile", || Profile::new(remain_args.clone()).run()),
        Arg::action(None, "setup", || Setup::new(remain_args.clone()).run()),
        Arg::action(None, "run", || Run::new(remain_args.clone()).run()),
        Arg::action(None, "", || Run::new(remain_args.clone()).run()),
        Arg::action(Some("-h"), "--help", || print_help(app_name())),
        Arg::action(Some("-V"), "--version", || {
            Ok(println!("{}", env!("CARGO_PKG_VERSION")))
        }),
    ];

    parse_into_vars("", &mut rules, cmd_deque)
        .strict_first()
        .ok()
}

/// Main entry point for the ALPack application.
///
/// This function centralizes error management and exit code reporting.
/// It ensures that any errors propagated through the logic are displayed
/// to the user without technical traces, while returning a standard
/// exit code 1 for failures to ensure compatibility with shell scripts.
fn main() {
    let exit_code: i32 = match alpack() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("{}", e);
            1
        }
    };
    std::process::exit(exit_code);
}
