mod apk;
mod aports;
mod builder;
mod command;
mod config;
mod mirror;
mod run;
mod setup;
mod settings;
mod utils;

use crate::apk::Apk;
use crate::aports::Aports;
use crate::builder::Builder;
use crate::config::Config;
use crate::run::Run;
use crate::setup::Setup;

use pico_args::Arguments;
use std::env;
use std::error::Error;

fn print_help(cmd: &str) -> Result<(), Box<dyn Error>> {
    println!("{cmd} - Alpine Linux RootFS Packaging Tool

ALPack is a simple shell-based tool that allows you
to create and manage Alpine Linux rootfs containers
easily using proot or bubblewrap(bwrap).

Usage:
    {cmd} <parameters> [options] [--] [ARGS...]

Available parameters:
        setup                   Initialize or configure the rootfs environment
        run                     Execute command inside the rootfs
        config                  Display or modify global configuration
        aports                  Manage local aports repositories
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
        --minimal               Install only the minimal set of packages
        --mirror <URL>          Use the specified mirror instead of the default one
        --mirror=<URL>          Use the specified mirror instead of the default one (inline)
        --cache <DIR>           Specify cache directory
        --cache=<DIR>           Specify cache directory (inline)
    -R, --rootfs <DIR>          Specify rootfs directory
        --rootfs=<DIR>          Specify rootfs directory (inline)

Options for 'apk':
    -R, --rootfs <DIR>          Specify rootfs directory
        --rootfs=<DIR>          Specify rootfs directory (inline)

Options for 'aports':
    -u, --update                Update the local aports repository to the latest version
    -s, --search=<PKG>          Search for a package in the Alpine aports
    -g, --get=<PKG>             Download the APKBUILD in the Alpine aports
    -R, --rootfs <DIR>          Specify rootfs directory
        --rootfs=<DIR>          Specify rootfs directory (inline)

Options for 'builder':
    -a, --apkbuild <APKBUILD>   Use a specific APKBUILD file as input
        --apkbuild=<APKBUILD>   Use a specific APKBUILD file as input (inline)
    -R, --rootfs <DIR>          Specify rootfs directory
        --rootfs=<DIR>          Specify rootfs directory (inline)

Options for 'run':
    -0, --root                  Run with root privileges inside rootfs
    -i, --ignore-extra-binds    Ignore additional bind mounts
    -b, --bind-args <ARGS>      Additional bind arguments (can be inline or next argument)
        --bind-args=<ARGS>      Additional bind arguments (inline)
    -c, --command <CMD>         Command to execute inside rootfs (can be repeated)
        --command=<CMD>         Command to execute (inline)
    -R, --rootfs <DIR>          Specify rootfs directory
        --rootfs=<DIR>          Specify rootfs directory (inline)

Options for 'config':
        --use-proot             Use 'proot' as rootfs handler (default)
        --use-bwrap             Use 'bwrap' as rootfs handler
        --use-latest-stable     Use 'latest-stable' release (default)
        --use-edge              Use 'edge' release
        --cache-dir <DIR>       Set cache directory
        --cache-dir=<DIR>       Set cache directory (inline)
        --output-dir <DIR>      Set output directory (default current directory)
        --output-dir=<DIR>      Set output directory (inline)
        --rootfs-dir <DIR>      Set rootfs directory
        --rootfs-dir=<DIR>      Set rootfs directory (inline)
        --default-mirror <URL>  Set default Alpine mirror
        --default-mirror=<URL>  Set default Alpine mirror (inline)

Global Options:
    -h, --help                  Show this help message
    -V, --version               Show version

Environment variables:
    ALPACK_ARCH       Define the target architecture for rootfs (e.g., x86_64, aarch64)
    ALPACK_ROOTFS     Specify the path to the root filesystem used by ALPack
    ALPACK_CACHE      Specify the path to the cache directory used by ALPack

Examples:
    {cmd} setup --rootfs=/mnt/alpine --minimal --edge
    {cmd} apk --rootfs=/mnt/alpine install curl
    {cmd} run -R /mnt/alpine -0 -- fdisk -l
");
    Ok(())
}

/// alpack is the main logic function of the program, returning a Result for error handling
fn alpack() -> Result<(), Box<dyn Error>> {
    let cmd = env::current_exe().unwrap().file_name().unwrap().to_str().unwrap().to_string();

    let mut pargs = Arguments::from_env();
    let command: Option<String> = pargs.opt_free_from_str().unwrap_or_default();

    let remaining_args: Vec<String> = pargs.finish().into_iter()
        .map(|s| s.into_string().unwrap_or_else(|os| os.to_string_lossy().into()))
        .collect();

    match command.as_deref() {
        Some("apk") => {
            let mut args = remaining_args.into_iter();
            let (mut rootfs, mut subcommand) = (None, None);
            let mut subargs: Vec<String> = Vec::new();

            while let Some(arg) = args.next() {
                if arg == "--rootfs" || arg == "-R" {
                    rootfs = args.next();
                } else if arg.starts_with("--rootfs=") {
                    rootfs = Some(arg.trim_start_matches("--rootfs=").to_string());
                } else if subcommand.is_none() {
                    subcommand = Some(arg);
                } else {
                    subargs.push(arg);
                }
            }

            let apk = Apk::new(cmd, subcommand, subargs, rootfs);
            apk.run()?;
            Ok(())
        },
        Some("add") | Some("del") | Some("install") | Some("remove") | Some("-s") |
        Some("search") | Some("update") | Some("fix") | Some("-u") => {
            let apk = Apk::new(cmd, command, remaining_args, None);
            apk.run()?;
            Ok(())
        },
        Some("aports") => {
            let aports = Aports::new(cmd, remaining_args);
            aports.run()?;
            Ok(())
        },
        Some("builder") => {
            let builder = Builder::new(cmd, remaining_args);
            builder.run()?;
            Ok(())
        },
        Some("config") => {
            let config = Config::new(cmd, remaining_args);
            config.run()?;
            Ok(())
        },
        Some("run") => {
            let run = Run::new(cmd, remaining_args);
            run.run()?;
            Ok(())
        },
        Some("setup") => {
            let mut setup = Setup::new(cmd, remaining_args);
            setup.run()?;
            Ok(())
        },
        Some("-h") | Some("--help") => {
            print_help(&cmd)?;
            Ok(())
        },
        Some("-V") | Some("--version") => {
            let version = env!("CARGO_PKG_VERSION");
            println!("{cmd} {version}");
            Ok(())
        },
        Some(other) => {
            Err(format!("{cmd}: invalid argument '{other}'\nUse '{cmd} --help' to see available options.").into())
        },
        None => {
            let run = Run::new(cmd, remaining_args);
            run.run()?;
            Ok(())
        }
    }
}

/// Main function with manual error handling to suppress automatic error messages
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
