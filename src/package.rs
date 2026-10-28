use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result, bail};
use jiff::Zoned;
use semver::Version;
use toml_edit::{DocumentMut, value};

use crate::changes::Changes;
use crate::level::Level;

const BOOKMARK: &str = "main";
const CARGO_TOML: &str = "Cargo.toml";
const CHANGES_MD: &str = "CHANGES.md";

/// The package to release, in the directory of its Cargo.toml.
pub struct Package {
    root: PathBuf,
}

impl Package {
    /// The package of the current directory, as cargo finds it.
    pub fn locate() -> Result<Package> {
        let output = Command::new("cargo")
            .args(["locate-project", "--message-format", "plain"])
            .output()
            .context("could not run cargo")?;
        if !output.status.success() {
            bail!("there is no Cargo.toml in this directory or above it");
        }
        let manifest_path = PathBuf::from(String::from_utf8(output.stdout)?.trim());
        let root = manifest_path
            .parent()
            .context("Cargo.toml has no directory")?
            .to_path_buf();
        Ok(Package { root })
    }

    pub fn release(&self, level: Option<Level>) -> Result<()> {
        if !self
            .output("jj", &["diff", "--name-only"])?
            .trim()
            .is_empty()
        {
            bail!("the working copy has changes; commit them first");
        }

        let cargo_toml_path = self.root.join(CARGO_TOML);
        let mut cargo_toml: DocumentMut = fs::read_to_string(&cargo_toml_path)?.parse()?;
        let name = cargo_toml["package"]["name"]
            .as_str()
            .context("Cargo.toml has no package name")?
            .to_string();
        let current = cargo_toml["package"]["version"]
            .as_str()
            .context("Cargo.toml has no package version")?
            .parse::<Version>()?;

        let changes_path = self.root.join(CHANGES_MD);
        let changes = Changes::new(
            fs::read_to_string(&changes_path)
                .with_context(|| format!("could not read {}", changes_path.display()))?,
        );
        let version = if let Some(level) = level {
            level.bump(&current)
        } else {
            if changes.has_releases() {
                bail!("give the level of the release: major, minor or patch");
            }
            current
        };
        let version = &version;
        let released = changes.release(version, Zoned::now().date())?;

        cargo_toml["package"]["version"] = value(version.to_string());
        fs::write(&cargo_toml_path, cargo_toml.to_string())?;
        fs::write(&changes_path, released.text())?;
        self.run("cargo", &["update", "--workspace"])?;

        // Verify the package before anything is committed. The working copy
        // was clean, so restoring it undoes only the release's own changes.
        if let Err(error) = self.run(
            "cargo",
            &["publish", "--package", &name, "--dry-run", "--allow-dirty"],
        ) {
            self.run("jj", &["restore"])?;
            return Err(error.context("the dry run failed; the version bump was undone"));
        }

        let tag = format!("v{version}");
        self.run(
            "jj",
            &["commit", "--message", &format!("Release {version}.")],
        )?;
        self.run("cargo", &["publish", "--package", &name, "--no-verify"])
            .context("publishing failed after the release commit was made")?;
        self.run("jj", &["tag", "set", &tag, "--revision", "@-"])?;
        self.run("jj", &["bookmark", "set", BOOKMARK, "--revision", "@-"])?;
        self.run(
            "jj",
            &["git", "push", "--bookmark", BOOKMARK, "--tag", &tag],
        )?;

        fs::write(&changes_path, released.with_unreleased().text())?;
        println!("Released {name} {version}. CHANGES.md has a new `## Unreleased` heading.");
        Ok(())
    }

    fn command(&self, program: &str, args: &[&str]) -> Command {
        let mut command = Command::new(program);
        command.args(args).current_dir(&self.root);
        command
    }

    fn run(&self, program: &str, args: &[&str]) -> Result<()> {
        let status = self
            .command(program, args)
            .status()
            .with_context(|| format!("could not run {program}"))?;
        if !status.success() {
            bail!("`{program} {}` failed", args.join(" "));
        }
        Ok(())
    }

    fn output(&self, program: &str, args: &[&str]) -> Result<String> {
        let output = self
            .command(program, args)
            .output()
            .with_context(|| format!("could not run {program}"))?;
        if !output.status.success() {
            bail!(
                "`{program} {}` failed: {}",
                args.join(" "),
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        Ok(String::from_utf8(output.stdout)?)
    }
}
