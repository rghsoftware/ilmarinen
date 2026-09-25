//! Fixture: the smallest Rust crate that exercises the Ilmarinen gates.

// specscore:implements feature/greeting#req:greet-by-name
pub fn greet(name: &str) -> String {
    format!("Hello, {name}!")
}

#[cfg(test)]
mod tests {
    use super::greet;

    // specscore:verifies feature/greeting#ac:greets-named-user
    #[test]
    fn greets_named_user() {
        assert_eq!(greet("Ada"), "Hello, Ada!");
    }
}
