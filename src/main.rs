use std::{process::ExitCode, time::Duration};

use clap::Parser;
use diverter::{Steam, Username};

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Sets to the account of USERNAME.
    #[arg(short, long, value_name = "USERNAME")]
    set: Option<Username>,
    /// Prints the current account.
    /// If used with --set, the original value will be printed.
    #[arg(short, long)]
    get: bool,
    #[arg(short, long, action = clap::ArgAction::Count)]
    /// Restarts Steam, with the switched user if supplied.
    /// If supplied twice, allows Steam to check file integrity on startup.
    /// If supplied three times, also shuts down Steam gracefully.
    restart: u8,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    if cli.get {
        match Steam::get_auto_login_user() {
            Ok(username) => println!("{username}"),
            Err(e) => {
                eprintln!("failed to get the current username: {e}");
                if cli.set.is_none() && cli.restart == 0 {
                    return ExitCode::from(69);
                }
            }
        }
    }

    if let Some(new_username) = cli.set {
        if let Err(e) = Steam::set_auto_login_user(new_username) {
            eprintln!("failed to set the new username: {e}");
            return ExitCode::from(69);
        }
    }

    if cli.restart > 0 {
        let steam = Steam::new().unwrap();
        let graceful_shutdown = cli.restart >= 3;
        let graceful_launch = cli.restart >= 2;
        if graceful_shutdown {
            steam.shutdown().unwrap();
            // wait for the original Steam process to close:
            std::thread::sleep(Duration::from_secs(10)); // XXX poll instead of wait
        } else {
            steam.kill().unwrap();
        }

        if graceful_launch {
            steam.launch().unwrap();
        } else {
            steam.launch_fast().unwrap();
        }
    }

    ExitCode::SUCCESS
}
