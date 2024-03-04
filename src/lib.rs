use std::path::PathBuf;

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

fn find_files(paths: &[String]) -> MyResult<Vec<PathBuf>> {
    unimplemented!()
}

pub fn run(cli: Cli) -> MyResult<()> {
    print!("{:#?}", cli);

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::find_files;

    #[test]
    fn test_find_files() {
        let res = find_files(&["./tests/inputs/jokes".to_string()]);
        assert!(res.is_ok());

        let files = res.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(
            files.first().unwrap().to_string_lossy(),
            "./tests/inputs/jokes"
        );

        // 存在しないファイルは失敗する
        let res = find_files(&["/path/does/not/exist".to_string()]);
        assert!(res.is_err());

        // 拡張子が .dat 以外の入力ファイルをすべて検索する
        let res = find_files(&["./tests/inputs".to_string()]);
        assert!(res.is_ok());
        let files = res.unwrap();
        assert_eq!(files.len(), 5);
        let first = files.first().unwrap().display().to_string();
        assert!(first.contains("ascii-art"));
        let last = files.last().unwrap().display().to_string();
        assert!(last.contains("quotes"));

        // 複数のソースに対するテスト
        // パスは重複無しでソートされた状態である
        let res = find_files(&[
            "./tests/inputs/jokes".to_string(),
            "./tests/inputs/ascii-art".to_string(),
            "./tests/inputs/jokes".to_string(),
        ]);
        assert!(res.is_ok());
        let files = res.unwrap();
        assert_eq!(files.len(), 2);
        if let Some(filename) = files.first().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "ascii-art".to_string());
        }
        if let Some(filename) = files.last().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "jokes".to_string());
        }
    }
}
