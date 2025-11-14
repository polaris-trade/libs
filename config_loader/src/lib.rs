pub type Hello = String;

// trigger a build
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = Hello::from("Hello, world!");
        assert_eq!(result, "Hello, world!");
    }
}
