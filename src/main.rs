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
mod help;
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
use crate::help::HELP_RULES;
use crate::overlay::Overlay;
use crate::profile::Profile;
use crate::run::Run;
use crate::settings::{Settings, settings_cmd};
use crate::setup::Setup;
use flexiargs::{Arg, NULL_PTR, ParserOptions, parse_into_vars};
use sandbox_utils::{sandbox_init, set_sandbox_tool};
use std::collections::VecDeque;
use std::env;
use std::error::Error;
use std::path::PathBuf;

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
        Arg::action(None, NULL_PTR, || Run::new(remain_args.clone()).run()),
    ];

    let opts = ParserOptions {
        subcommand: NULL_PTR,
        help_rules: HELP_RULES,
        ..Default::default()
    };

    parse_into_vars(&mut rules, cmd_deque, opts)
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
