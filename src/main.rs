use std::{io::Read, process::ExitCode, time::Duration};

use clap::Parser;
use diverter::{vdf, Steam, Username};

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
    /// Print with color. Leave unspecified for auto.
    #[arg(short, long)]
    color: Option<bool>,
}

#[derive(Debug, Clone, Copy, clap::Subcommand)]
enum Command {
    #[command(alias = "g")]
    /// Prints the current account.
    Get,
    /// Sets to the account of USERNAME.
    #[command(alias = "s")]
    Set {
        /// The username of the account to switch to.
        username: Username,
        #[arg(short, long)]
        /// Restart the Steam client ungracefully after setting the new user.
        restart: bool,
        /// Restarts the Steam client gracefully after setting the new user.
        ///
        /// Implies --restart.
        #[arg(short, long)]
        graceful: bool,
        /// After restart, allows Steam to verify file integrity.
        ///
        /// Implies --restart.
        #[arg(short, long)]
        verify: bool,
    },
    /// Lists registered Steam users.
    #[command(alias = "l", alias = "ls")]
    List,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Command::Get => match Steam::get_auto_login_user() {
            Ok(username) => println!("{username}"),
            Err(e) => eprintln!("Error: {e}"),
        },
        Command::Set {
            username,
            restart,
            graceful,
            verify,
        } => {
            if let Err(e) = Steam::set_auto_login_user(username) {
                eprintln!("Failed to set the new username: {e}");
                return ExitCode::from(&e);
            }
            if restart || graceful || verify {
                match Steam::new() {
                    Ok(steam) => {
                        let (kill_method, kill_method_verb, kill_result) = if graceful {
                            (
                                "shut down",
                                "shut down",
                                steam.shutdown_poll(Duration::from_millis(100)),
                            )
                        } else {
                            ("killed", "kill", steam.kill().map(|_| ()))
                        };

                        match kill_result {
                                Ok(()) => eprintln!("Steam has been {kill_method}"),
                                Err(e) => eprintln!("Failed to {kill_method_verb} Steam to restart it ({e}). Will still try to launch it..")
                            }

                        let launch_result = if verify {
                            steam.launch()
                        } else {
                            steam.launch_fast()
                        };
                        match launch_result {
                            Ok(()) => eprintln!("Launched Steam"),
                            Err(e) => {
                                eprintln!("Failed to re-launch Steam: {e}");
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to find Steam to restart it: {e}");
                    }
                }
            }
        }
        Command::List => match Steam::new() {
            Ok(steam) => match steam.vdf_loginusers() {
                Ok(mut vdf_file) => {
                    let should_color = cli.color.unwrap_or_else(|| atty::is(atty::Stream::Stdout));
                    let mut vdf_source = String::with_capacity(4096);
                    if let Err(e) = vdf_file.read_to_string(&mut vdf_source) {
                        eprintln!("Failed to read logged in users data: {e}");
                        return ExitCode::from(69);
                    }

                    match vdf::scan_parse(vdf_source.as_bytes()) {
                        Ok(document) => match vdf::LoginUser::from_vdf(&document) {
                            Ok(login_users) => {
                                let existing_username = Steam::get_auto_login_user().ok();
                                let existing_username = existing_username
                                    .as_ref()
                                    .map(|username| username.as_bytes());

                                login_users.for_each(|user| match user {
                                    Ok(user) => {
                                        let selected = Some(user.username) == existing_username;
                                        println!(
                                            "{ansi_start}{} {} ({}){ansi_end}",
                                            if selected { "*" } else { "-" },
                                            user.username.escape_ascii(),
                                            user.nickname.escape_ascii(),
                                            ansi_start = if should_color && selected {
                                                "\u{1B}[32m"
                                            } else {
                                                ""
                                            },
                                            ansi_end = if should_color { "\u{1B}[0m" } else { "" },
                                        )
                                    }
                                    Err(e) => eprintln!("failed to read user entry: {e}"),
                                });
                            }
                            Err(e) => {
                                eprintln!("Failed to parse logged in users data: {e}");
                                return ExitCode::from(69);
                            }
                        },
                        Err(e) => {
                            eprintln!("Failed to parse logged in users data: {e}");
                            return ExitCode::from(69);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to find logged in users data: {e}");
                    return ExitCode::from(&e);
                }
            },
            Err(e) => {
                eprintln!("Failed to find Steam: {e}");
                return ExitCode::from(&e);
            }
        },
    }

    ExitCode::SUCCESS
}
