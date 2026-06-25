# atom Development Harness

## What this repository is

atom (Anything to Markdown) is a Rust CLI / API that converts documents
into readable, structured Markdown.

For anything documented under `docs/`, read the source of truth rather
than duplicating it here:

- Requirements: `docs/requirements.md`
- Implementation plan: `docs/implementation-plan.md`
- Tasks, per-format scope, and current status: `docs/tasks.md`

## Improvement loop

1. Run `just test` to check the tests.
2. Run `just eval` to inspect the conversion report JSON.
3. Triage a failing fixture or warning into one of: input parser, AST,
   writer, or evaluation function.
4. Apply the smallest fix.
5. Run `just ci` to confirm there is no regression.

## Do not

- Do not rewrite `tests/fixtures/**/*.expected.md` without justification.
- Do not lower evaluation-function thresholds to hide failures.
- Do not enable external LLM sending by default.
- Do not add confidential documents to fixtures.
