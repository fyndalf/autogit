# autogit

Don't you hate having to navigate to every single git repository in your `projects` folder and update them all by hand?

`autogit`, a small CLI tool built with rust, aims to solve this!
Simply run `autogit` in the root `projects` folder (or wherever you have your git repos), and it will take a look at each repo and update it if possible.

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

## Build from source

```bash
git clone git@github.com:fyndalf/autogit.git
cargo build --release
```

### Dependencies
- quicli
- git2
- human-panic
- console
- indicatif

## To Dos