# Normalize Punctuation

![Crates.io License](https://img.shields.io/crates/l/normalize-punctuation)
![GitHub Tag](https://img.shields.io/github/v/tag/qrichert/normalize-punctuation?sort=semver&filter=*.*.*&label=release)
[![crates.io](https://img.shields.io/crates/d/normalize-punctuation?logo=rust&logoColor=white&color=orange)](https://crates.io/crates/normalize-punctuation)
[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/qrichert/normalize-punctuation/ci.yml?label=tests)](https://github.com/qrichert/normalize-punctuation/actions)

_A highly opinionated punctuation normalizer for Markdown files._

## Get `--help`

```
A highly opinionated punctuation normalizer for Markdown files.

Usage: normalize-punctuation [OPTIONS] [PATH ...]

Arguments:
  [PATH ...]  Markdown files or directories to scan [default: current directory]

Options:
  -h, --help       Show this message and exit.
  -V, --version    Show the version and exit.
```

## Philosophy

This tool canonicalizes Markdown source; it is not a typesetting engine.

Typographic punctuation variants are normalized to keyboard-friendly
ASCII. The inverse transformation is deliberately not attempted:
choosing the right curly quote or apostrophe requires linguistic context
that cannot be reliably inferred from Markdown alone.

Unicode spaces are handled differently because their width and
line-breaking behavior carry meaning. NBSP, NNBSP, thin space, and
regular space can be indistinguishable in ordinary editors, and their
rendered differences are often difficult to spot. Replacing Unicode
spaces with explicit HTML character references makes them visible in
source and diffs, giving authors and reviewers a chance to identify and
fix unintended spacing.

The tool never inserts spaces. It only rewrites spaces already present
in the source. An existing thin space before `;`, `?`, or `!` becomes a
narrow non-breaking space; one before `:` becomes a non-breaking space,
following French (France) conventions. Text without such a space, such
as `Hello!`, is left unchanged.

Normalization is text-based and also applies inside Markdown code spans
and code blocks. This is an intentional scope tradeoff: those cases are
rare, and the tool favors simple, consistent normalization over
Markdown-aware exceptions.

## Current replacements

- `‘` → `'`
- `’` → `'`
- `“` → `"`
- `”` → `"`
- `ˋ` → `` ` ``
- `‚` → `'`
- `„` → `"`
- `…` → `...`
- `NBSP` (`U+00A0`) → `&nbsp;`
- `NNBSP` (`U+202F`) → `&#8239;`
- `THIN SPACE` (`U+2009`) before `;`, `?`, or `!` → `&#8239;`
- `THIN SPACE` (`U+2009`) before `:` → `&nbsp;`
- `THIN SPACE` (`U+2009`) otherwise → `&thinsp;`
- `«` followed by `SPACE`, `NBSP`, `NNBSP`, or `THIN SPACE` → `"`
- `«` → `"`
- `»` preceded by `SPACE`, `NBSP`, `NNBSP`, or `THIN SPACE` → `"`
- `»` → `"`
- `‐` → `-`
- `﹘` → `-`
- `−` → `-`
- `–` → `-`

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

Install from [crates.io] with Cargo:

```shell
cargo install normalize-punctuation
```

Pre-built binaries for Linux and macOS are available on the [latest GitHub release].

[Documentation] is available on docs.rs.

[crates.io]: https://crates.io/crates/normalize-punctuation
[latest GitHub release]: https://github.com/qrichert/normalize-punctuation/releases/latest
[Documentation]: https://docs.rs/normalize-punctuation
