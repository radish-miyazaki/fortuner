use clap::Parser;
use regex::{Regex, RegexBuilder};

type MyResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Parser, Debug)]
#[command(
    name = "fortuner",
    version = "0.1.0",
    author = "Radish-Miyazaki <y.hidaka.kobe@gmail.com>",
    about = "Rust fortune"
)]
pub struct Cli {
    #[arg(
        value_name = "FILE",
        help = "Input files or directories",
        required = true
    )]
    sources: Vec<String>,
    #[arg(value_name = "PATTERN", help = "Pattern", short = 'm', long)]
    pattern: Option<Regex>,
    #[arg(value_name = "SEED", help = "Random seed", short, long)]
    seed: Option<u64>,
    #[arg(
        help = "Case-insensitive pattern matching",
        short,
        long,
        default_value = "false"
    )]
    insensitive: bool,
}

pub fn get_cli() -> MyResult<Cli> {
    let mut cli = Cli::parse();

    if let Some(patten) = cli.pattern {
        cli.pattern = RegexBuilder::new(&patten.to_string())
            .case_insensitive(cli.insensitive)
            .build()
            .map(Some)
            .map_err(|e| e.to_string())?;
    };

    Ok(cli)
}

pub fn run(cli: Cli) -> MyResult<()> {
    print!("{:#?}", cli);

    Ok(())
}
