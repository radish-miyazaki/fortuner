use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{BufRead, BufReader},
    path::PathBuf,
};

use clap::Parser;
use rand::{rngs::StdRng, seq::SliceRandom, thread_rng, SeedableRng};
use regex::{Regex, RegexBuilder};
use walkdir::WalkDir;

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
    let mut files = vec![];

    for path in paths {
        match fs::metadata(path) {
            Err(e) => Err(format!("{}: {}", path, e))?,
            Ok(_) => {
                WalkDir::new(path)
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter(|e| {
                        e.file_type().is_file() && e.path().extension() != Some(OsStr::new("dat"))
                    })
                    .for_each(|e| files.push(e.path().to_path_buf()));
            }
        }
    }

    files.sort();
    files.dedup();
    Ok(files)
}

#[derive(Debug)]
pub struct Fortune {
    source: String,
    text: String,
}

fn read_fortunes(paths: &[PathBuf]) -> MyResult<Vec<Fortune>> {
    let mut fortunes: Vec<Fortune> = vec![];
    let mut buffer = vec![];

    for path in paths {
        let source = path.file_name().unwrap().to_string_lossy().to_string();
        let file = File::open(path).map_err(|e| format!("{}: {}", source, e))?;

        for line in BufReader::new(file).lines().map_while(Result::ok) {
            if line != "%" {
                buffer.push(line.to_string());
                continue;
            }

            if !buffer.is_empty() {
                fortunes.push(Fortune {
                    source: source.clone(),
                    text: buffer.join("\n"),
                });
                buffer.clear();
            }
        }
    }

    Ok(fortunes)
}

fn pick_fortune(fortunes: &[Fortune], seed: Option<u64>) -> Option<String> {
    let fortune = match seed {
        Some(seed) => fortunes.choose(&mut StdRng::seed_from_u64(seed)),
        None => fortunes.choose(&mut thread_rng()),
    }?;

    Some(fortune.text.clone())
}

pub fn run(cli: Cli) -> MyResult<()> {
    let files = find_files(&cli.sources)?;
    let fortunes = read_fortunes(&files)?;

    if let Some(pattern) = cli.pattern {
        let mut prev_source = None;

        for fortune in fortunes {
            if pattern.is_match(&fortune.text) {
                if prev_source.as_ref().map_or(true, |s| s != &fortune.source) {
                    eprintln!("({})\n%", fortune.source);
                    prev_source = Some(fortune.source.clone());
                }

                println!("{}\n%", fortune.text);
            }
        }
    } else {
        let fortune = pick_fortune(&fortunes, cli.seed);
        println!(
            "{}",
            fortune.unwrap_or_else(|| "No fortunes found".to_string())
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{find_files, pick_fortune, read_fortunes, Fortune};

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

    #[test]
    fn test_read_fortunes() {
        // 入力ファイルが1つだけの場合
        let res = read_fortunes(&[PathBuf::from("./tests/inputs/jokes")]);
        assert!(res.is_ok());

        if let Ok(fortunes) = res {
            assert_eq!(fortunes.len(), 6);
            assert_eq!(
                fortunes.first().unwrap().text,
                "Q. What do you call a head of lettuce in a shirt and tie?\n\
                A. Collared greens."
            );
            assert_eq!(
                fortunes.last().unwrap().text,
                "Q: What do you call a deer wearing an eye patch?\n\
                A: A bad idea (bad-eye deer)."
            );
        }

        // 入力ファイルが複数の場合
        let res = read_fortunes(&[
            PathBuf::from("./tests/inputs/jokes"),
            PathBuf::from("./tests/inputs/quotes"),
        ]);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 11);
    }

    #[test]
    fn test_pick_fortune() {
        let fortunes = &[
            Fortune {
                source: "fortunes".to_string(),
                text: "You cannot achieve the impossible without \
                attempting the absurd."
                    .to_string(),
            },
            Fortune {
                source: "fortunes".to_string(),
                text: "Assumption is the mother of all screw-apps.".to_string(),
            },
            Fortune {
                source: "fortunes".to_string(),
                text: "Neckties strangle clear thinking.".to_string(),
            },
        ];

        assert_eq!(
            pick_fortune(fortunes, Some(1)).unwrap(),
            "Neckties strangle clear thinking.",
        );
    }
}
