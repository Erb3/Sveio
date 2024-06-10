use clap::{command, Parser};

#[derive(Parser)]
#[command(name = "Sveio", version, about, author)]
pub struct Cli {
	/// Optional port to use. Default is 8085
	#[arg(short, long, env = "SVEIO_PORT")]
	pub port: Option<u32>,
}

pub fn get_settings() -> Cli {
	Cli::parse()
}
