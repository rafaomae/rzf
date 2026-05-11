mod algo;
mod ui;
mod walk;

use std::path::Path;

use crate::ui::State;

fn main() -> std::io::Result<()> {
    let candidates: Vec<String> = walk::walk_dir(Path::new("."))?;

    let mut state = State::new(candidates);
    if let Some(selection) = ui::run(&mut state)? {
        println!("{selection}");
    }

    Ok(())
}
