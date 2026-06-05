//! Environment setup orchestration.
//!
//! This module handles the initial preparation of the Alpine Linux environment,
//! including mirror selection, version discovery, rootfs extraction, and
//! provisioning of default packages.

use crate::help::HELP_RULES;
use crate::mirror::Mirror;
use crate::settings::{settings_cache_dir, settings_profile, settings_rootfs_dir};
use flexiargs::{Arg, ParserOptions, parse_into_vars};
use regex::Regex;
use sandbox_utils::{
    ArchiveConfig, SandBox, SandBoxConfig, app_arch, app_name, download_file, extract_bootstrap,
    map_result, set_profile, success_finish_setup, temp_cache,
};
use scraper::{Html, Selector};
use std::collections::VecDeque;
use std::error::Error;
use std::fs;

/// Structured version components for semantic comparison.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VersionKey {
    major: u32,
    minor: u32,
    patch: u32,
    suffix: String,
}

/// Default packages installed when minimal mode is disabled.
pub const DEF_PACKAGES: &str =
    "alpine-sdk autoconf automake cmake glib-dev glib-static libtool go xz";

/// Controller for setting up the Alpine Linux rootfs environment.
pub struct Setup {
    /// Command line arguments not consumed by the main parser.
    args: Vec<String>,
}

impl Setup {
    /// Creates a new `Setup` instance.
    pub fn new(args: Vec<String>) -> Self {
        Self { args }
    }

    /// Orchestrates the setup process including version discovery and installation.
    ///
    /// This method parses setup-specific flags, identifies the latest available
    /// minirootfs on the selected mirror, and executes the extraction and
    /// initial package setup via `apk`.
    ///
    /// # Returns
    /// - `Ok(())` on successful environment initialization.
    /// - `Err` if any stage (download, extraction, or execution) fails.
    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let args: VecDeque<String> = self.args.iter().cloned().collect();
        let mut profile = settings_profile();
        let mut use_mirror: Option<String> = None;
        let (mut no_cache, mut reinstall, mut edge, mut minimal) = (false, false, false, false);
        let (mut cache_dir, mut rootfs) = (settings_cache_dir(), settings_rootfs_dir());

        let mut rules = [
            Arg::bool(None, "--edge", &mut edge),
            Arg::bool(Some("-m"), "--minimal", &mut minimal),
            Arg::bool(None, "--no-cache", &mut no_cache),
            Arg::bool(Some("-r"), "--reinstall", &mut reinstall),
            Arg::option(None, "--mirror", "url", &mut use_mirror),
            Arg::value(Some("-R"), "--rootfs", "directory", &mut rootfs),
            Arg::value(Some("-p"), "--profile", "profile", &mut profile),
            Arg::value(None, "--cache", "directory", &mut cache_dir),
        ];

        let opts = ParserOptions {
            subcommand: "setup",
            help_rules: HELP_RULES,
            ..Default::default()
        };

        if parse_into_vars(&mut rules, args, opts).help_or_err()? {
            return Ok(());
        }

        drop(rules);

        let prof_resolved = set_profile(&profile);
        let profile_path = rootfs.join(&prof_resolved);

        if !reinstall && profile_path.exists() && profile_path.is_dir() {
            return Err(format!(
                "Rootfs directory '{}' is already available.\nUse [-r|--reinstall] to reinstall it.",
                profile_path.display()
            ).into());
        }

        if reinstall && profile_path.exists() {
            println!("Reinstalling directory '{}'", profile_path.display());
            obliterate::ensure_removed(&profile_path)?;

            let upper_path = rootfs.join(format!("{}_upper", prof_resolved));
            if upper_path.exists() {
                println!("Cleaning associated overlay upper: '{}'", upper_path.display());
                obliterate::ensure_removed(&upper_path)?;
            }
        }

        if no_cache {
            cache_dir = temp_cache();
        }

        let mut mirror = Mirror::new(use_mirror, edge.then_some("edge".to_string()));
        mirror.run()?;

        let url = mirror.get_mirror();
        let res = ureq::get(&url).call()?.body_mut().read_to_string()?;

        let document = Html::parse_document(&res);
        let selector = Selector::parse("a")?;

        let pattern = format!(r"^alpine-minirootfs-([\w.\-]+)-{}\.tar\.gz$", app_arch());
        let re = Regex::new(&pattern)?;

        let mut matches = vec![];
        for element in document.select(&selector) {
            if let Some(href) = element.value().attr("href") {
                if let Some(caps) = re.captures(href) {
                    let version_str = &caps[1];
                    if let Some(key) = Self::parse_version_key(version_str) {
                        matches.push((key, version_str.to_string(), href.to_string()));
                    }
                }
            }
        }

        matches.sort_by(|a, b| a.0.cmp(&b.0));

        if let Some((_, version, link)) = matches.last() {
            println!("Latest version found: {version}");
            println!("Link: {url}{link}");
            download_file(&format!("{url}{link}"), cache_dir.clone(), link)?;
            extract_bootstrap(
                cache_dir.join(link),
                profile_path.clone(),
                ArchiveConfig::default(),
            )?;

            if no_cache {
                let _ = fs::remove_dir_all(&cache_dir);
            }

            let repo_path = profile_path.join("etc/apk/repositories");
            fs::write(&repo_path, mirror.get_repository())?;

            let apk_command = if minimal {
                "apk update && apk upgrade".to_string()
            } else {
                format!("apk update && apk upgrade && apk add {DEF_PACKAGES}")
            };

            let config = SandBoxConfig {
                rootfs,
                profile,
                run_cmd: apk_command,
                use_root: true,
                ignore_extra_bind: true,
                ..Default::default()
            };

            map_result(SandBox::run(config))?;
        } else {
            Err("No alpine-minirootfs files found")?;
        }

        success_finish_setup(format!("{} run", app_name()).as_str())
    }

    /// Parses a version string into a `VersionKey` struct.
    ///
    /// # Arguments
    /// * `link_contain_version` - A string slice containing the version string to parse.
    ///
    /// # Returns
    /// * `Some(VersionKey)` if the string is successfully parsed.
    /// * `None` if the string does not match the expected version pattern.
    fn parse_version_key(link: &str) -> Option<VersionKey> {
        let re = Regex::new(r"^(\d+)\.(\d+)\.(\d+)(?:[_\-]?([a-zA-Z0-9]+))?$").ok()?;
        let caps = re.captures(link)?;

        Some(VersionKey {
            major: caps.get(1)?.as_str().parse().ok()?,
            minor: caps.get(2)?.as_str().parse().ok()?,
            patch: caps.get(3)?.as_str().parse().ok()?,
            suffix: caps
                .get(4)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default(),
        })
    }
}
