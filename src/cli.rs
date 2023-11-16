use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Starting port to listen on
    #[arg(short, long, default_value = "8080")]
    pub port: usize,

    /// Number of ports to listen on (port + count)
    #[arg(short, long, default_value = "10")]
    pub count: usize,

    /// Country code to use for exit node
    #[arg(short, long)]
    pub exit_country: Option<String>,
}

pub fn parse() -> Cli {
    Cli::parse()
}
