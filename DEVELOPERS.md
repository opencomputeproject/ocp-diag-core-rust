### Coding style

- unit tests should return a `Result` type. Example:
    ```rust
    #[test]
    fn is_equal_to_42() -> anyhow::Result<()> {
        let x = maybe_return_42()?;
        assert_eq!(x, 42);
        Ok(())
    }
    ```