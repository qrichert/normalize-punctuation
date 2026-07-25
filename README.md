# Normalize Punctuation

![Crates.io License](https://img.shields.io/crates/l/normalize-punctuation)
![GitHub Tag](https://img.shields.io/github/v/tag/qrichert/normalize-punctuation?sort=semver&filter=*.*.*&label=release)
[![crates.io](https://img.shields.io/crates/d/normalize-punctuation?logo=rust&logoColor=white&color=orange)](https://crates.io/crates/normalize-punctuation)
[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/qrichert/normalize-punctuation/ci.yml?label=tests)](https://github.com/qrichert/normalize-punctuation/actions)

_Normalize punctuation in Markdown files._

## Get `--help`

```
Normalize punctuation in Markdown files.

Usage: normalize-punctuation [OPTIONS] [PATH ...]

Arguments:
  [PATH ...]  Markdown files or directories to scan [default: current directory]

Options:
  -h, --help       Show this message and exit.
  -V, --version    Show the version and exit.
```

## Current replacements

- `‘` → `'`
- `’` → `'`
- `“` → `"`
- `”` → `"`
- `ˋ` → `` ` ``
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
- `–` → `-`

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
