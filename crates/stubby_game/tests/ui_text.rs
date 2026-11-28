//! The game draws its text in Bevy's default font, a small subset of Fira Mono: a
//! character outside it shows as an empty box. "·" and "−" did so in the menus and "·"
//! again on the crafting screen (2026-11-27). On-screen text is ASCII.

use std::path::Path;

/// Non-ASCII characters inside string literals of the game's code, outside comments,
/// entity names (`Name::new`, never drawn) and the window title (drawn by the system).
#[test]
fn text_on_screen_is_ascii() {
    let mut found = Vec::new();
    for crate_dir in [
        "stubby_game",
        "stubby_ui",
        "stubby_streams",
        "bevy_stubby_rack",
    ] {
        let src = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join(crate_dir)
            .join("src");
        scan(&src, &mut found);
    }
    assert!(
        found.is_empty(),
        "on-screen text must be ASCII (the default font lacks the rest):\n{}",
        found.join("\n")
    );
}

fn scan(dir: &Path, found: &mut Vec<String>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            scan(&path, found);
            continue;
        }
        if path.extension().is_none_or(|e| e != "rs") || path.ends_with("main.rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        for (n, line) in text.lines().enumerate() {
            let code = line.trim_start();
            if code.starts_with("//") || line.contains("Name::new") {
                continue;
            }
            for literal in literals(line) {
                if !literal.is_ascii() {
                    found.push(format!("{}:{}: {literal}", path.display(), n + 1));
                }
            }
        }
    }
}

/// The string literals on one line (ignoring a comment after the code).
fn literals(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '/' if chars.peek() == Some(&'/') => break,
            '\'' => {
                // A char literal ('"' among them), or a lifetime: skip to its end.
                let rest: String = chars.clone().take(3).collect();
                if rest.starts_with('\\') {
                    chars.nth(2);
                } else if rest.chars().nth(1) == Some('\'') {
                    chars.nth(1);
                }
            }
            '"' => {
                let mut literal = String::new();
                while let Some(c) = chars.next() {
                    match c {
                        '\\' => {
                            chars.next();
                        }
                        '"' => break,
                        c => literal.push(c),
                    }
                }
                out.push(literal);
            }
            _ => {}
        }
    }
    out
}
