use clap::{command, Parser};

#[derive(clap::ValueEnum, Clone, Debug)]
pub(crate) enum LoggingLevel {
	Trace,
	Debug,
	Info,
	Warn,
	Error,
	None,
}

impl From<LoggingLevel> for tracing_subscriber::filter::LevelFilter {
	fn from(value: LoggingLevel) -> Self {
		match value {
			LoggingLevel::Trace => tracing_subscriber::filter::LevelFilter::TRACE,
			LoggingLevel::Debug => tracing_subscriber::filter::LevelFilter::DEBUG,
			LoggingLevel::Info => tracing_subscriber::filter::LevelFilter::INFO,
			LoggingLevel::Warn => tracing_subscriber::filter::LevelFilter::WARN,
			LoggingLevel::Error => tracing_subscriber::filter::LevelFilter::ERROR,
			LoggingLevel::None => tracing_subscriber::filter::LevelFilter::OFF,
		}
	}
}

#[derive(Parser)]
#[command(name = "Sveio", version, about, author)]
pub(crate) struct Cli {
	/// Optional port to use. Default is 8085
	#[arg(short, long, env = "SVEIO_PORT")]
	pub(crate) port: Option<u32>,

	/// Optional logging level to use. Default is info
	#[arg(short, long, env = "SVEIO_LOGGING_LEVEL")]
	pub(crate) logging: Option<LoggingLevel>,

	/// Optional amount of seconds to allow guessing. Default is 7s
	#[arg(long, env = "SVEIO_GUESS_TIME")]
	pub(crate) guess_time: Option<u64>,

	/// Optional amount of seconds where players can see where the others
	/// guessed. Default is 3s
	#[arg(long, env = "SVEIO_SHOWCASE_TIME")]
	pub(crate) showcase_time: Option<u64>,
}

pub(crate) fn get_settings() -> Cli {
	Cli::parse()
}
