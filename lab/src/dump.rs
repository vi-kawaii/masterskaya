use arboard::Clipboard;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub fn run(cwd: &Path) -> Result<(), String> {
    let include_ext = [
        "rs", "toml", "md", "json", "yaml", "yml", "js", "ts", "tsx", "jsx", "html", "css",
        "sql", "sh", "ron", "wgsl", "txt", "xml", "lock",
    ];
    let skip_dirs = ["target", ".git", "node_modules", "dist", "build", ".idea"];

    let mut output = String::new();

    // 1. Дерево
    output.push_str("=== PROJECT TREE ===\n");
    for entry in WalkDir::new(cwd)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !skip_dirs.contains(&name.as_ref())
        })
        .filter_map(|e| e.ok())
    {
        let depth = entry.depth();
        let name = entry.file_name().to_string_lossy();
        let prefix = "  ".repeat(depth);
        output.push_str(&format!("{prefix}{name}\n"));
    }

    // 2. Содержимое
    output.push_str("\n=== FILE CONTENTS ===\n");
    for entry in WalkDir::new(cwd)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !skip_dirs.contains(&name.as_ref())
        })
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        if !include_ext.contains(&ext) {
            continue;
        }
        let display = path.strip_prefix(cwd).unwrap_or(path);
        output.push_str(&format!("\n── {} ──\n", display.display()));
        match fs::read_to_string(path) {
            Ok(content) => output.push_str(&content),
            Err(_) => output.push_str("<не удалось прочитать файл>\n"),
        }
        output.push('\n');
    }

    // 3. В буфер
    match Clipboard::new() {
        Ok(mut cb) => match cb.set_text(output.clone()) {
            Ok(_) => {
                println!("✅ Контекст скопирован в буфер ({} KB)", output.len() / 1024);
                Ok(())
            }
            Err(e) => Err(format!("не могу записать в буфер: {e}")),
        },
        Err(e) => Err(format!("нет доступа к буферу: {e}")),
    }
}
