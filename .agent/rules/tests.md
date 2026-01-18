---
trigger: always_on
---

- When writing unit tests, only use googletest matchers, never bare `assert!` or `assert_eq!`.
- When writing unit tests, do not start test names with `test_`, but rather give a descriptive name in a form like "these_inputs_make_those_outputs", etc.
- When running unit tests, only use `mise run tests`, never `cargo test` directly.
