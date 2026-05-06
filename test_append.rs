fn main() {
    let line = "docs-architecture.md";
    let mut current = String::new();
    let mut in_whitespace = None;
    for ch in line.chars() {
        let is_ws = ch.is_whitespace();
        if in_whitespace == Some(is_ws) || in_whitespace.is_none() {
            current.push(ch);
            in_whitespace = Some(is_ws);
            continue;
        }
        println!("Token: {}", current);
        current.clear();
        current.push(ch);
        in_whitespace = Some(is_ws);
    }
    if !current.is_empty() {
        println!("Token: {}", current);
    }
}
