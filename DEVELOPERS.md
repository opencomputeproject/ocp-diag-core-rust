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

### Release process

To make a new release, and publish to crates.io, a new tagged commit needs to exist on the `main` branch. This is done with a simple merge from the `dev` branch. **Do not** push any other kinds of commits to the `main` branch.

Steps:
1. bump the version. Will need [`cargo-release`](https://crates.io/crates/cargo-release) crate. Example here bumps the *patch* version.
```bash
$ git checkout dev
$ cargo release version patch --execute
$ cargo release changes  # note any changelog to add to the commit, or manually craft it
$ git add .
$ git commit
$ git push origin dev
```
2. merge `dev` into `main`
```bash
$ git checkout main
$ git merge --no-ff dev
```
3. tag the merge commit
```bash
$ git checkout main
$ cargo release tag --sign-tag --execute
```
4. push with tags
```bash
$ git checkout main
$ git push
$ git push --tags
```
