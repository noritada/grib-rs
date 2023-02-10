# Development

## Releasing

Currently, the release process is a solo operation. Although it really should be automated, there is still a huge amount of development work that needs to be done on the software itself, and the priority for automation is much lower than for the development of the software itself, so the release process remains a manual operation for now.

Since there is a lot of work that needs to be done in the release process, and I have often forgotten some of the tasks, I have been writing down the steps in a messy memo so that I don't forget. However, I decided to create this document because it is better to write down the steps in a proper document than to clutter up the working directory with such a messy memo.

1. Update `CHANGELOG.md`.
   The output of `git log --oneline $(git describe --tags --abbrev=0)..HEAD`, the tree view of `gitk` and a series of PRs are the source of the content that will be written in `CHANGELOG.md`. However, it takes a lot of time to compile the necessary information while ignoring unnecessary chores.
2. Bump the version number by editing `Cargo.toml`s of packages.
   Note that the version information is also included in the dependency descriptions since packages in the crate have dependencies.
3. Commit changes above in the `master` branch.
4. Switch to a clean repository and pull the `master` branch.
   Cloning is better to avoid mistakes, but fast-forward updating of an existing repository is fine as long as it stays clean, so I always have a clean repository for the release operation.
5. Run `cargo test --workspace --release` to reconfirm that the tests pass.
6. Run `cargo doc --no-deps --open` in lib crate to reconfirm the generated documentation.
7. Run the following commands, starting with the dependent packages, i.e. `grib-build`, `grib`, `grib-cli`, in that order.
   - `cargo package`
   - `cargo publish`
8. Switch to the original repository and tag the release by running following commands.
   - `git tag v0.0.0` (of course, it is necessary to change the tag name according to the version number.)
   - `git push`
   - `git push --tags`
