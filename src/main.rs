use std::process;

use clap::Parser;
use diverter::{Steam, Username};

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Switches to the account of USERNAME.
    #[arg(short, long, value_name = "USERNAME")]
    switch: Option<Username>,
    /// Gets the current account.
    #[arg(short, long)]
    get: bool,
    #[arg(short, long, requires = "switch")]
    /// Restarts Steam with the switched user.
    restart: bool,
}

fn main() {
    let cli = Cli::parse();
    let steam = Steam::new().unwrap();

    if cli.get {
        match Steam::get_auto_login_user() {
            Ok(username) => {
                println!("{}", std::str::from_utf8(username.as_bytes()).expect("the retrieved string of the account from the registry is not valid ASCII/UTF-8."));
            }
            Err(e) => {
                eprintln!("failed to get the current username: {e}");
                process::exit(1)
            }
        }
    }

    if let Some(new_username) = cli.switch {
        if let Err(e) = Steam::set_auto_login_user(new_username) {
            eprintln!("failed to set the new username: {e}");
            process::exit(1)
        }
    }

    if cli.restart {
        steam.kill().unwrap();
        steam.launch_fast().unwrap();
    }
}
