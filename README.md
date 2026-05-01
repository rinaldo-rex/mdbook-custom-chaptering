# mdbook-custom-chaptering

An mdbook preprocessor plugin for custom chapter numbering.

Eg: I created this for using it as a calver (Calendar versioning) based chapter numbering

## Rationale: 
I vibe-coded this because I needed a quick little solution. Use with caution

## Features

- **Custom chapter numbers** using dot notation (e.g., `26.5`, `2026.5.1`)
- **No auto-incrementing** - whatever number you specify in SUMMARY.md appears in the sidebar
- **Enable/disable** via `book.toml` configuration

## Installation

### From Crates.io (coming soon)

```bash
cargo install mdbook-custom-chaptering
```

### From Git

```bash
cargo install --git https://github.com/yourusername/mdbook-custom-chaptering
```

Or clone and build:

```bash
git clone https://github.com/yourusername/mdbook-custom-chaptering.git
cd mdbook-custom-chaptering
cargo build --release
cp target/release/mdbook-custom-chaptering ~/.cargo/bin/
```

## Usage

### 1. Add to `book.toml`

```toml
[preprocessor.custom-chaptering]
enabled = true
renderers = ["html"]
```

### 2. Use in `SUMMARY.md`

Write chapters with custom numbers using dot notation:

```markdown
# Summary

- [26.5 First chapter](./first.md)
  - [5. Some month](./month.md)
    - [1. Day within month](./day1.md)
    - [8. Another day](./day8.md)
```

### 3. Build the book

```bash
mdbook build
```

## Result

The rendered HTML sidebar will display:

```
26.5. First chapter
5. Some month
1. Day within month
8. Another day
```

## Configuration Options

| Option | Type | Default | Description |
|--------|------|--------|-------------|
| `enabled` | bool | `true` | Enable/disable the preprocessor |
| `renderers` | array | all | List of renderers to run on |

Example:

```toml
[preprocessor.custom-chaptering]
enabled = true
renderers = ["html"]  # Only run with HTML renderer
```

## Requirements

- mdBook 0.5.0+
- Rust 1.70+

## License

MIT
