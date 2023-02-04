# <img src="https://raw.githubusercontent.com/yehuthi/diverter/master/doc/diverter.png" alt="diverter" width=100 align=left /> diverter [<img src="https://img.shields.io/crates/v/diverter" align="right" />](https://crates.io/crates/diverter)

Switch between Steam accounts without relogging on Windows.

## Usage

Typical usage is as follows:
```shell
diverter -rs my_other_account # change account and restart Steam
```

The `s[et] <USERNAME>` flag sets the account by username, and `r[estart]` indicates a restart.

The restart flag can be supplied multiple times, each time makes the restart slower but more graceful. If supplied three times, it will shut down Steam gracefully, which is a good idea for when you think Steam might be in the middle of a filesystem operation, such as when you're downloading a game, uploading your save to the Steam Cloud, etc.

```shell
diverter -rrrs my_other_account # change account and restart Steam gracefully
```

See `--help` for complete usage documentation.

# Installation

Download [the latest release](https://github.com/yehuthi/diverter/releases/latest), or build using [cargo](https://www.rust-lang.org/tools/install) from source via `cargo install diverter`.
