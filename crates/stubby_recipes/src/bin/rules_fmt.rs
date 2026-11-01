//! Put the rules files in canonical form, regenerate their JSON Schemas, add new IDs
//! to `ids.lock`, then validate and report every problem.
//!
//! `cargo run -p stubby_recipes --bin rules-fmt [-- <rules dir>]`
//! (default: `assets/recipes` in this repository). `--check` changes nothing and exits
//! non-zero when a file would change or a rule is invalid.
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use stubby_recipes::rules::{self, Rules, RulesErrors, RulesFiles};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let check = args.iter().any(|a| a == "--check");
    let dir = args
        .iter()
        .find(|a| !a.starts_with("--"))
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/recipes"));
    match run(&dir, check) {
        Ok(changed) if check && !changed.is_empty() => {
            eprintln!("not canonical: {}", changed.join(", "));
            eprintln!("run `cargo run -p stubby_recipes --bin rules-fmt`");
            ExitCode::FAILURE
        }
        Ok(changed) => {
            for file in changed {
                println!("wrote {file}");
            }
            println!("rules OK");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn read(dir: &Path, file: &str) -> Result<String, String> {
    std::fs::read_to_string(dir.join(file))
        .map_err(|e| format!("{}: {e}", dir.join(file).display()))
}

fn run(dir: &Path, check: bool) -> Result<Vec<String>, String> {
    let texts: Vec<String> = [
        rules::ITEMS,
        rules::RESOURCES,
        rules::MINES,
        rules::RECIPES,
        rules::MIGRATIONS,
    ]
    .iter()
    .map(|f| read(dir, f))
    .collect::<Result<_, _>>()?;
    let files = RulesFiles {
        items: &texts[0],
        resources: &texts[1],
        mines: &texts[2],
        recipes: &texts[3],
        migrations: &texts[4],
    };
    let mut outputs: Vec<(String, String)> = rules::canonical_files(&files)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|(f, t)| (f.to_owned(), t))
        .collect();
    outputs.extend(rules::schemas());
    let rules = Rules::load(&files).map_err(|e| e.to_string())?;
    let lock = std::fs::read_to_string(dir.join(rules::IDS_LOCK)).unwrap_or_default();
    outputs.push((
        rules::IDS_LOCK.into(),
        rules::updated_ids_lock(&lock, &rules),
    ));

    let mut changed = Vec::new();
    for (file, text) in outputs {
        let path = dir.join(&file);
        if std::fs::read_to_string(&path).ok().as_deref() != Some(text.as_str()) {
            if !check {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
            }
            changed.push(file);
        }
    }
    let lock = if check {
        lock
    } else {
        read(dir, rules::IDS_LOCK)?
    };
    let errors = rules::check_ids_lock(&lock, &rules);
    if !errors.is_empty() && !(check && changed.iter().any(|f| f == rules::IDS_LOCK)) {
        return Err(RulesErrors(errors).to_string());
    }
    Ok(changed)
}
