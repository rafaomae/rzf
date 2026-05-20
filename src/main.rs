mod fuzzy;
mod ui;
mod walk;

use std::io::{self, IsTerminal, Read};
use std::path::Path;

use crate::ui::State;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.len() == 2 && args[0] == "--filter" {
        return filter(&args[1]);
    }

    if !args.is_empty() {
        eprintln!("usage: rzf [--filter QUERY]");
        std::process::exit(2);
    }

    let candidates = load_candidates()?;

    let mut state = State::new(candidates);
    if let Some(selection) = ui::run(&mut state)? {
        println!("{selection}");
    }

    Ok(())
}

fn filter(query: &str) -> io::Result<()> {
    let candidates = load_candidates()?;

    let mut results: Vec<(i32, String)> = candidates
        .into_iter()
        .filter_map(|candidate| fuzzy::score(query, &candidate).map(|score| (score, candidate)))
        .collect();

    results.sort_by_key(|(score, _)| std::cmp::Reverse(*score));

    for (_, candidate) in results {
        println!("{candidate}");
    }

    Ok(())
}

fn load_candidates() -> io::Result<Vec<String>> {
    let mut stdin = io::stdin();

    if stdin.is_terminal() {
        return walk::walk_dir(Path::new("."));
    }

    let mut input = String::new();
    stdin.read_to_string(&mut input)?;
    Ok(candidates_from_input(&input))
}

fn candidates_from_input(input: &str) -> Vec<String> {
    input.lines().map(str::to_string).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_candidates_from_lines() {
        let candidates = candidates_from_input("src/main.rs\nCargo.toml\n");

        assert_eq!(candidates, vec!["src/main.rs", "Cargo.toml"]);
    }
}
