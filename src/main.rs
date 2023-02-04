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
                    return e.into();
                }
            }
        }
    }

    if let Some(new_username) = cli.set {
        if let Err(e) = Steam::set_auto_login_user(new_username) {
            eprintln!("failed to set the new username: {e}");
            return e.into();
        }
    }

    if cli.restart > 0 {
        let steam = match Steam::new() {
            Ok(steam) => steam,
            Err(e) => {
                eprintln!("failed to locate Steam for restarting: {e}");
                return e.into();
            }
        };
        let graceful_shutdown = cli.restart >= 3;
        let graceful_launch = cli.restart >= 2;
        let shutdown_result = if graceful_shutdown {
            let result = steam.shutdown();
            if result.is_ok() {
                // wait for Steam processes to close:
                loop {
                    if steam.is_running().unwrap() {
                        std::thread::sleep(Duration::from_millis(100));
                    } else {
                        break;
                    }
                }
            }
            result
        } else {
            steam.kill().map(|_| ())
        };

        if let Err(e) = shutdown_result {
            eprintln!("failed to shut Steam down in order to restart it (will still attempt to launch it): {e}");
        }

        let launch_result = if graceful_launch {
            steam.launch()
        } else {
            steam.launch_fast()
        };
        if let Err(e) = launch_result {
            eprintln!("failed to launch Steam to restart it: {e}");
            return e.into();
        }
    }

    ExitCode::SUCCESS
}
