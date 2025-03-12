# Normalize Punctuation

_Normalize punctuation in Markdown files._

## Current replacements

- `‘` → `'`
- `’` → `'`
- `“` → `"`
- `”` → `"`
- `‚` → `'`
- `„` → `"`
- `…` → `...`
- `NBSP` → ` `
- `« ` → `"`
- `«` → `"`
- ` »` → `"`
- `»` → `"`
- `‐` → `-`
- `﹘` → `-`
- `−` → `-`
- `–` → `—`

To keep `NBSP`s, use explicit `&nbsp;` instead.

<!-- - `NNBSP` → `` -->

## Installation

### As a `pre-commit` hook (recommended)

To run `normalize-punctuation` as a `pre-commit` hook, add the following
to your `.pre-commit-config.yaml` file:

```yaml
- repo: https://github.com/qrichert/normalize-punctuation
  rev: <tag of latest version>
  hooks:
    - id: normalize-punctuation
```

### As a standalone executable

```shell
cargo install --locked --git https://github.com/qrichert/normalize-punctuation.git
```

Use the same command to update `normalize-punctuation`.
