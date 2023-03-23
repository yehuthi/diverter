# <img src="https://raw.githubusercontent.com/yehuthi/diverter/master/doc/diverter.png" alt="diverter" width=100 align=left /> diverter [<img src="https://img.shields.io/crates/v/diverter" align="right" />](https://crates.io/crates/diverter)

Switch between Steam accounts without relogging on Windows.

## Usage

Typical usage is as follows:

```shell
diverter set my_other_account -r # change account and restart Steam
```

The `set <username>` command will have Steam attempt to log in to the other account on its next launch.
The `-r` / `--restart` means to restart Steam, starting the switch immediately.

`--restart` kills the Steam process ungracefully (see implications below), alternatively you can use `-g` / `--graceful` for a graceful restart. Additionally you can complement an ungraceful restart with the `-v` / `--verify` flag to allow Steam to verify files after it restarts.

```shell
diverter set my_other_account -r # restart ungracefully
diverter set my_other_account -g # restart gracefully
diverter set my_other_account -v # restart ungracefully but verify files
```

> Tip: Restarting Steam ungracefully is much quicker but can cause data corruption, so it's a good idea to restart gracefully when you think Steam might be in the middle of a filesystem operation, such as when you're downloading a game, uploading your save to the Steam Cloud, etc.

See `--help` for complete usage documentation.

# Installation

Download [the latest release](https://github.com/yehuthi/diverter/releases/latest), or build using [cargo](https://www.rust-lang.org/tools/install) from source via `cargo install diverter`.
