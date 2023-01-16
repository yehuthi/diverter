use std::process;

use clap::Parser;
use diverter::Username;

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Switches to the account of USERNAME.
    #[arg(short, long, value_name = "USERNAME")]
    switch: Option<Username>,
    /// Gets the current account.
    #[arg(short, long)]
    get: bool,
}

fn main() {
    let cli = Cli::parse();
    if cli.get {
        let mut buffer = [0u8; 33];
        match diverter::get_auto_login_user(&mut buffer) {
            Ok(len) => {
                eprintln!("{}", std::str::from_utf8(&buffer[..len]).expect("the retrieved string of the account from the registry is not valid ASCII/UTF-8."));
            }
            Err(e) => {
                eprintln!("failed to get the current username: {e}");
                process::exit(1)
            }
        }
    }
    if let Some(new_username) = cli.switch {
        if let Err(e) = diverter::set_auto_login_user(new_username.as_bytes_with_nul()) {
            eprintln!("failed to set the new username: {e}");
            process::exit(1)
        }
    }
}
