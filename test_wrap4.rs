fn get_terminal_wrapped_lines(line: &str, wrap_cols: usize) -> Vec<String> {
    if line.is_empty() {
        return vec![String::new()];
    }
    let chars: Vec<char> = line.chars().collect();
    let mut chunks = Vec::new();
    for chunk in chars.chunks(wrap_cols) {
        chunks.push(chunk.iter().collect());
    }
    chunks
}

fn main() {
    let line = "VIEW-architecture.md ci-cd-architecture.md docs-architecture.md";
    println!("{:#?}", get_terminal_wrapped_lines(line, 53));
}
