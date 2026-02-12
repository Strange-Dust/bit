# Build and Test

Run the full build, test, and lint pipeline. Use this after making code changes to verify everything works.

## Steps

1. Run `cargo build --release` to compile in release mode
2. Run `cargo test` to execute all unit and integration tests
3. Run `cargo clippy -- -D warnings` to check for lint issues
4. Report results: which steps passed/failed, and any errors or warnings

## On failure

- If build fails: show the compiler error and suggest a fix
- If tests fail: show which tests failed and the failure output
- If clippy warns: show the warnings and apply fixes
