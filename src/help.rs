//! Application help rules configuration.

use flexiargs::ArgHelp;

/// Global command-line help definitions for the ALPack utility.
pub static HELP_RULES: &[ArgHelp] = &[
    ArgHelp::properties(
        "Alpine Linux RootFS SandBox Packager",
        "Lightweight shell utility for managing Alpine Linux\nrootfs containers via proot or bwrap.",
        env!("CARGO_PKG_VERSION"),
    ),
    ArgHelp::subcommand(None, "setup", "Configure rootfs environment"),
    ArgHelp::subcommand(None, "run", "Execute command inside rootfs"),
    ArgHelp::subcommand(None, "config", "Manage global configuration"),
    ArgHelp::subcommand(None, "aports", "Manage local aports repository"),
    ArgHelp::subcommand(None, "aptree", "Manage Adélie package tree"),
    ArgHelp::subcommand(None, "builder", "Build packages and images"),
    ArgHelp::subcommand(None, "profile", "Manage configuration profiles"),
    ArgHelp::subcommand(None, "overlay", "Configure OverlayFS options"),
    ArgHelp::subcommand(None, "apk", "Run Alpine package manager"),
    ArgHelp::subcommand(None, "add", "Install packages into rootfs")
        .meta("<ARGS>"),
    ArgHelp::subcommand(None, "del", "Remove packages from rootfs")
        .meta("<ARGS>"),
    ArgHelp::subcommand(Some("-i"), "install", "Alias for 'add'")
        .meta("<ARGS>"),
    ArgHelp::subcommand(Some("-r"), "remove", "Alias for 'del'")
        .meta("<ARGS>"),
    ArgHelp::subcommand(Some("-s"), "search", "Search available packages")
        .meta("<ARGS>"),
    ArgHelp::subcommand(Some("-u"), "update", "Update indices and upgrade packages"),
    ArgHelp::subcommand(None, "fix", "Repair broken package installations"),
    ArgHelp::arg(Some("-R"), "--rootfs", "Specify rootfs directory")
        .meta("<PATH>")
        .context(&["setup", "apk", "aports", "aptree", "builder", "run", "profile", "overlay"]),
    ArgHelp::arg(Some("-p"), "--profile", "Specify profile name")
        .meta("<PROFILE>")
        .context(&["setup", "apk", "aports", "aptree", "builder", "run"]),
    ArgHelp::arg(Some("-r"), "--reinstall", "Reinstall rootfs environment")
        .context(&["setup"]),
    ArgHelp::arg(Some("-m"), "--minimal", "Install minimal package set")
        .context(&["setup"]),
    ArgHelp::arg(None, "--edge", "Use edge (testing) repository")
        .context(&["setup"]),
    ArgHelp::arg(None, "--mirror", "Set alpine mirror URL")
        .meta("<URL>")
        .context(&["setup"]),
    ArgHelp::arg(None, "--cache", "Define cache directory")
        .meta("<PATH>")
        .context(&["setup"]),
    ArgHelp::arg(None, "--no-cache", "Disable cache directory")
        .context(&["setup"]),
    ArgHelp::arg(Some("-u"), "--update", "Update local repository to latest")
        .context(&["aports", "aptree"]),
    ArgHelp::arg(Some("-s"), "--search", "Search for a package")
        .meta("<PKG...>")
        .context(&["aports", "aptree"]),
    ArgHelp::arg(Some("-S"), "--strict-search", "Search for exact package")
        .meta("<PKG...>")
        .context(&["aports", "aptree"]),
    ArgHelp::arg(Some("-g"), "--get", "Download APKBUILD file")
        .meta("<PKG...>")
        .context(&["aports", "aptree"]),
    ArgHelp::arg(Some("-o"), "--output", "Output directory for APKBUILD")
        .meta("<PATH>")
        .context(&["aports", "aptree"]),
    ArgHelp::arg(Some("-e"), "--ephemeral", "Run in ephemeral mode")
        .context(&["run", "builder"]),
    ArgHelp::arg(Some("-o"), "--overlay", "Enable overlay filesystem")
        .context(&["run", "builder"]),
    ArgHelp::arg(Some("-a"), "--apkbuild", "Use specific APKBUILD file")
        .meta("<APKBUILD...>")
        .context(&["builder"]),
    ArgHelp::arg(None, "--force-key", "Force RSA key regeneration")
        .context(&["builder"]),
    ArgHelp::arg(Some("-0"), "--root", "Run with root privileges in rootfs")
        .context(&["run"]),
    ArgHelp::arg(Some("-i"), "--ignore-extra-binds", "Ignore extra mounts")
        .context(&["run"]),
    ArgHelp::arg(Some("-s"), "--secure-rootfs", "Secure minimal mounts")
        .context(&["run"]),
    ArgHelp::arg(Some("-b"), "--bind-args", "Add custom bind arguments")
        .meta("<ARGS>")
        .context(&["run"]),
    ArgHelp::arg(Some("-k"), "--kill-on-exit", "Kill all child processes when the sandbox exits")
        .context(&["run"]),
    ArgHelp::arg(None, "--unshare-pid", "Isolate the sandbox into its own PID namespace (bwrap only)")
        .context(&["run"]),
    ArgHelp::arg(Some("-w"), "--pwd", "Set the initial working directory inside the sandbox")
        .meta("<PATH>")
        .context(&["run"]),
    ArgHelp::arg(None, "--cwd", "Set the initial working directory inside the sandbox")
        .meta("<PATH>")
        .context(&["run"]),
    ArgHelp::arg(Some("-c"), "--command", "Command to execute inside rootfs")
        .meta("<COMMANDS...>")
        .context(&["run"]),
    ArgHelp::arg(None, "--use-proot", "Use proot as rootfs handler (default)")
        .context(&["config"]),
    ArgHelp::arg(None, "--use-bwrap", "Use bwrap as rootfs handler")
        .context(&["config"]),
    ArgHelp::arg(None, "--mirror-stable", "Use latest-stable release (default)")
        .context(&["config"]),
    ArgHelp::arg(None, "--mirror-edge", "Use edge release")
        .context(&["config"]),
    ArgHelp::arg(None, "--cache-dir", "Set cache directory")
        .meta("<PATH>")
        .context(&["config"]),
    ArgHelp::arg(None, "--output-dir", "Set output directory (default current)")
        .meta("<PATH>")
        .context(&["config"]),
    ArgHelp::arg(None, "--rootfs-dir", "Set rootfs directory")
        .meta("<PATH>")
        .context(&["config"]),
    ArgHelp::arg(None, "--profile", "Set profile name")
        .meta("<PROFILE>")
        .context(&["config"]),
    ArgHelp::arg(None, "--default-mirror", "Set default Alpine mirror")
        .meta("<URL>")
        .context(&["config"]),
    ArgHelp::arg(None, "--use-overlay", "Enable OverlayFS layer")
        .context(&["config"]),
    ArgHelp::arg(None, "--enable-overlay", "Enable OverlayFS layer")
        .context(&["config"]),
    ArgHelp::arg(None, "--disable-overlay", "Disable OverlayFS usage (default)")
        .context(&["config"]),
    ArgHelp::arg(None, "--use-persistent-inode", "Use persistent inodes for overlay")
        .context(&["config"]),
    ArgHelp::arg(None, "--use-virtual-inode", "Use virtual inodes for overlay (default)")
        .context(&["config"]),
    ArgHelp::arg(None, "--overlay-action-discard", "Discard changes on session end")
        .context(&["config"]),
    ArgHelp::arg(None, "--overlay-action-commit", "Commit changes back to rootfs")
        .context(&["config"]),
    ArgHelp::arg(None, "--overlay-action-commit-atomic", "Commit changes atomically")
        .context(&["config"]),
    ArgHelp::arg(None, "--overlay-action-preserve", "Preserve upper layer data", )
        .context(&["config"]),
    ArgHelp::arg(None, "--clean-cache", "Clean cache directory")
        .context(&["config"]),
    ArgHelp::arg(None, "--reset-config", "Reset configuration to defaults")
        .context(&["config"]),
    ArgHelp::arg(None, "--purge", "Purge all data")
        .context(&["config"]),
    ArgHelp::arg(Some("-p"), "--set-profile", "Set active profile")
        .meta("<PROFILE>")
        .context(&["profile", "overlay"]),
    ArgHelp::arg(None, "--no-confirm", "Skip confirmation prompts")
        .context(&["config", "profile", "overlay"]),
    ArgHelp::arg(Some("-r"), "--remove", "Remove the specified profile")
        .context(&["profile"]),
    ArgHelp::arg(None, "--rename", "Rename the active profile")
        .meta("<NEW_NAME>")
        .context(&["profile"]),
    ArgHelp::arg(None, "--default", "Set the current profile as default")
        .context(&["profile"]),
    ArgHelp::arg(None, "--ask", "Interactively choose what to do with the upper layer")
        .context(&["overlay"]),
    ArgHelp::arg(None, "--commit", "Commit upper layer changes into the rootfs")
        .context(&["overlay"]),
    ArgHelp::arg(None, "--commit-atomic", "Commit upper layer changes atomically into the rootfs")
        .context(&["overlay"]),
    ArgHelp::arg(None, "--discard", "Discard all upper layer changes")
        .context(&["overlay"]),
    ArgHelp::arg(None, "--force-files", "Force include specific paths in the commit")
        .meta("<PATH...>")
        .context(&["overlay"]),
    ArgHelp::arg(None, "--skip-dirs", "Skip specific directories during commit")
        .meta("<PATH...>")
        .context(&["overlay"]),
    ArgHelp::arg(None, "--skip-files", "Skip files matching specific names globally")
        .meta("<NAME...>")
        .context(&["overlay"]),
    ArgHelp::arg(None, "--skip-regex", "Skip files matching a regex pattern")
        .meta("<PATTERN...>")
        .context(&["overlay"]),
    ArgHelp::arg(None, "--skip-glob", "Skip files matching a glob pattern")
        .meta("<PATTERN...>")
        .context(&["overlay"]),
    ArgHelp::arg(None, "--skip-empty-in", "Skip empty files inside specific directories")
        .meta("<PATH...>")
        .context(&["overlay"]),
    ArgHelp::arg(None, "--skip-homedir", "Exclude home directory changes from the commit")
        .context(&["overlay"]),
    ArgHelp::arg(None, "--skip-rootdir", "Exclude root directory changes from the commit")
        .context(&["overlay"]),
    ArgHelp::arg(None, "--skip-zero-perms", "Exclude files with 000 permissions from the commit")
        .context(&["overlay"]),
    ArgHelp::arg(None, "--no-rootfs-preset", "Disable the default rootfs preset filter rules")
        .context(&["overlay"]),
    ArgHelp::env("ALPACK_ARCH", "Define the target architecture for rootfs"),
    ArgHelp::env("ALPACK_ROOTFS", "Specify the path to the root filesystem"),
    ArgHelp::env("ALPACK_CACHE", "Specify the path to the cache directory"),
];
