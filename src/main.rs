mod changes;
mod level;
mod package;

use anyhow::Result;
use clap::{Args, Parser};

use level::Level;
use package::Package;

/// Cargo runs a subcommand `cargo jj-release` as `cargo-jj-release
/// jj-release`, so the subcommand comes first.
#[derive(Parser)]
#[command(name = "cargo", bin_name = "cargo")]
enum CargoCommand {
    JjRelease(ReleaseArgs),
}

/// Release the next version of the package in the current directory: bump
/// Cargo.toml, stamp CHANGES.md, publish to crates.io, commit and tag with jj,
/// and push to GitHub.
#[derive(Args)]
#[command(version)]
struct ReleaseArgs {
    /// The level of the release. Leave it out for the first release, to
    /// release the current version.
    level: Option<Level>,
}

fn main() -> Result<()> {
    let CargoCommand::JjRelease(release_args) = CargoCommand::parse();
    Package::locate()?.release(release_args.level)
}
