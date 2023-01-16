use clap::Parser;
use diverter::Username;

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Switches to the account of USERNAME.
    #[arg(short, long, value_name = "USERNAME")]
    switch: Username,
}

fn main() {
    let cli = Cli::parse();
    diverter::set_auto_login_user(cli.switch.as_bytes_with_nul()).unwrap();
}
