# autogit

[![Build Status](https://travis-ci.com/fyndalf/autogit.svg?token=pzxEnLQoVdYwoArquwFZ&branch=master)](https://travis-ci.com/fyndalf/autogit)

Don't you hate having to navigate to every single git repository in your `projects` folder and update them all by hand?

`autogit`, a small CLI tool built with rust, aims to solve this!
Simply run `autogit` in the root `projects` folder (or wherever you have your git repos), and it will take a look at each repo and update it if possible.

Please note that for now, `git` needs to be installed on your machine.

## Usage

```
autogit 0.1.0
Finn K <fyndalf@users.noreply.github.com>
Update all git repositories that are located in subfolders

USAGE:
    autogit [FLAGS] [OPTIONS]

FLAGS:
    -f, --force      Force resetting and updating of currently checked out branches
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -d, --depth <depth>    How deep to check for git repositories [default: 3]
```

## Installation

```bash
curl -LSfs https://japaric.github.io/trust/install.sh | \
sh -s -- --git fyndalf/autogit
```

## Build from source

```bash
git clone git@github.com:fyndalf/autogit.git
cargo build --release
```

For development,
```
cargo fmt
cargo clippy
```
is also encouraged.

### Dependencies
- [quicli](https://github.com/killercup/quicli)
- [git2](https://github.com/rust-lang/git2-rs)
- [structop](https://github.com/TeXitoi/structopt)
- [human-panic](https://github.com/rust-cli/human-panic)
- [console](https://github.com/mitsuhiko/console)
- [indicatif](https://github.com/mitsuhiko/indicatif)
- [trust](https://github.com/japaric/trust)

## To Dos
- [ ] Add some tests
- [ ] Use git2 for fetching and pulling instead of bash-ed git commands