# Contributing to the `grib` library

All contributions including bug reports, bug fixes, documentation improvements, feature enhancements, and ideas are always welcome.

## Where to start?

Since this project is still in the early stage, there are so many things to do that I cannot write them down.  However, if multiple contributors do the same work, both of them will get unhappy.  So, it is helpful if you first make an issue before you start working so that the project can assign you.

When this project can grow to have a good community and continuous contributors, major themes will be settled and more descriptions will be written down.

## Setting up the development environment

There are two key points to note when setting up the development environment for this project:

- Be sure to check out the repositories included in this repository as Git submodules.
- Use the versions of the [`rustfmt`][rustfmt] formatter and [`rust-clippy`][rust-clippy] linter included in the nightly toolchain.

If you understand what to do based on the instructions above, you do not need to read the details below.

### Details

First, this repository uses the submodules functionality of Git. So, before running `cargo build`, please add submodules in one of following ways:

* Cloning the main repository with submodules:
  adding `--recursive` to `git clone` will automatically clone submodules in addition to this repository
* Checking out submodules after cloning the main repository:
  running `git submodule update --init --recursive` after cloning will update the repository to have submodules

For the Rust development environment, a standard environment set up using [`rustup`][rustup] or a similar tool should suffice. For `rustfmt` and `rust-clippy`, we use the nightly versions (some settings in `rustfmt.toml` require the nightly version). For example, if you are using `rustup`, you can install them as follows:

```
rustup toolchain install nightly --component rustfmt clippy
```

[rustup]: https://rustup.rs/
[rustfmt]: https://github.com/rust-lang/rustfmt
[rust-clippy]: https://github.com/rust-lang/rust-clippy

## Specifying dependencies

When using other crates, we have a policy that we do not designate dependency versions too strictly.

Based on [semantic versioning](https://semver.org/), for stable crates with a major version of 1 or higher, we specify dependencies using only the major version as much as possible (i.e., `"1"` for version `"x.y.z"`). Also, for unstable crates whose major version is less than 1, we specify dependencies with minor versions as much as possible (i.e., `"0.y"` for `"0.y.z"`).

Of course, this is not the case if we need to be more specific in order to avoid bugs or API changes in a particular version.

See the [Cargo documentation](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html) for specifying dependencies.

## Before committing

### Formatting and linting

When running formatters or linters, be sure to use the nightly version and target your workspace, as shown below.

```
cargo +nightly fmt --all
cargo +nightly clippy --workspace
```

### Testing

Testing is very important. Please run tests before committing.

You can test all packages in the workspace using this simple command:

```
cargo test --workspace
```

Since passing tests needs successful building, the prerequisites for committing are now all fulfilled.

## Commit message

As is commonly known, Git uses the first line as a subject line.  So, please use a following style if you want to set longer commit messages:

```
<subject line>
<BLANK LINE>
<body>
```

For more details, please check past commit messages.  That is the shortest route.
