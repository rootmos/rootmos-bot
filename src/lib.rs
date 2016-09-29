pub mod hello {
    pub fn hello_string() -> String {
        "Hello, library world!".to_string()
    }

    #[test]
    fn hello_string_is_a_greeting() {
        assert!(hello_string().to_lowercase().contains("hello"));
    }

    #[test]
    fn hello_string_mentions_world() {
        assert!(hello_string().to_lowercase().contains("world"));
    }
}
