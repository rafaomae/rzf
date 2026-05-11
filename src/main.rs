mod algo;
mod ui;
mod walk;

use std::{path::Path};

use crate::ui::State;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();


    if args.len() == 2 && args[0] == "--filter" {
        return filter(&args[1]);
    }

    if !args.is_empty() {
        eprintln!("usage: rzf [--filter QUERY]");
        std::process::exit(2);
    }

    let candidates: Vec<String> = walk::walk_dir(Path::new("."))?;

    let mut state = State::new(candidates);
    if let Some(selection) = ui::run(&mut state)? {
        println!("{selection}");
    }

    Ok(())
}

 fn filter(query: &str) -> std::io::Result<()> {
      let candidates = walk::walk_dir(Path::new("."))?;

      let mut results: Vec<(i32, String)> = candidates
          .into_iter()
          .filter_map(|candidate| {
              algo::score(query, &candidate).map(|score| (score, candidate))
          })
          .collect();

      results.sort_by_key(|(score, _)| std::cmp::Reverse(*score));

      for (_, candidate) in results {
          println!("{candidate}");
      }

      Ok(())
  }
