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
        let len = diverter::get_auto_login_user(&mut buffer).unwrap();
        println!("{}", std::str::from_utf8(&buffer[..len]).unwrap());
    }
    if let Some(new_username) = cli.switch {
        diverter::set_auto_login_user(new_username.as_bytes_with_nul()).unwrap();
    }
}
