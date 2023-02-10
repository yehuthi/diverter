use std::{io::Read, process::ExitCode, time::Duration};

use clap::Parser;
use diverter::{vdf, Steam, Username};

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
    /// Lists registered Steam users.
    #[arg(short, long)]
    list: bool,
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

    let mut steam: Option<Result<Steam, ()>> = None;
    let mut lazy_steam = || match steam {
        Some(Ok(steam)) => Ok(steam),
        Some(Err(())) => Err(()),
        None => match Steam::new() {
            Ok(s) => {
                steam = Some(Ok(s));
                Ok(s)
            }
            Err(e) => {
                steam = Some(Err(()));
                eprintln!("failed to find Steam: {e}");
                Err(())
            }
        },
    };

    if cli.list {
        if let Ok(steam) = lazy_steam() {
            let mut source = String::new();
            match steam.vdf_loginusers() {
                Ok(mut vdf_file) => match vdf_file.read_to_string(&mut source) {
                    Ok(_) => {
                        let document =
                            vdf::parse(vdf::Scanner::new(source.as_bytes()).map(|x| x.unwrap()))
                                .unwrap();
                        vdf::LoginUser::from_vdf(&document)
                            .unwrap()
                            .for_each(|user| {
                                println!(
                                    "{} ({})",
                                    user.unwrap().username.escape_ascii(),
                                    user.unwrap().nickname.escape_ascii()
                                )
                            });
                    }
                    Err(e) => eprintln!("failed to read loginusers.vdf (for --list): {e}"),
                },
                Err(e) => eprintln!("failed to open loginusers.vdf (for --list): {e}"),
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
        if let Ok(steam) = lazy_steam() {
            let graceful_shutdown = cli.restart >= 3;
            let graceful_launch = cli.restart >= 2;
            let shutdown_result = if graceful_shutdown {
                let result = steam.shutdown();
                if result.is_ok() {
                    // wait for Steam processes to close:
                    loop {
                        match steam.is_running() {
                            Ok(true) => std::thread::sleep(Duration::from_millis(100)),
                            Ok(false) => break,
                            Err(e) => {
                                eprintln!("failed to check if Steam closed, which is necessary for a graceful shutdown: {e}");
                                return e.into();
                            }
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
        } else {
            eprintln!("skipping --restart (Steam wasn't found)")
        }
    }

    ExitCode::SUCCESS
}
