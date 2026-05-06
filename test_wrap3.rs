fn chunk_string(s: &str, size: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();
    for c in s.chars() {
        if current.chars().count() == size {
            chunks.push(current.clone());
            current.clear();
        }
        current.push(c);
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

fn main() {
    let s = "docs-architecture.md  target";
    println!("{:?}", chunk_string(s, 9));
}
