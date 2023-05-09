use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "Akatsuki PP", version = "0.1.0")]
pub struct Args {
    #[clap(short, long, default_value = "config.toml")]
    pub config: String,
}

pub fn parse() -> Args {
    Args::parse()
}
