use thread_lens::models::ThreadDump;
use thread_lens::parser::parse_jstack_output;
use std::fs;
use std::path::Path;

pub fn read_dumps_from_directory(dir_path: String) -> std::io::Result<Vec<ThreadDump>> {
    let mut dumps = Vec::new();
    let path = Path::new(&dir_path);

    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "jstack") {
                let content = fs::read_to_string(&path)?;
                match parse_jstack_output(&content) {
                    Ok(dump) => dumps.push(dump),
                    Err(e) => eprintln!("Error parsing file {}: {}", path.display(), e),
                }
            }
        }
    }
    Ok(dumps)
}
