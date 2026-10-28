# cargo-jj-release

Release a Rust crate from a [jj] repository.

Install it:

```
cargo install cargo-jj-release
```

The crate keeps a `CHANGES.md`, written by hand, with the changes of the next
release under an `## Unreleased` heading:

```
# Changes

## Unreleased

- New thing.

## 0.1.0 (2026-10-27)

- Initial release.
```

With the changes committed and in main, release from the directory of the crate:

```
cargo jj-release <level>
```

where `<level>` is `major`, `minor` or `patch`. For the first release, leave
out the level to release the version that is already in `Cargo.toml`.

It automatically bumps the version in `Cargo.toml` and updates `Cargo.lock`,
does a dry-run and undoes the release changes if it fails, and commits
the release version with `jj`. It also tags the release commit `v<version>`
and moves the `main` bookmark.

An empty `## Unreleased` section is added to `CHANGES.md`, for you to commit.

[jj]: https://github.com/jj-vcs/jj
