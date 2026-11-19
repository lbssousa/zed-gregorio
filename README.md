# Gregorio GABC — Zed Extension

> [!NOTE]
> **AI-Assisted Development**
>
> This project is developed, fully or in part, with LLM/generative AI assistance.
> All LLM-proposed changes go through human review and local testing before being committed.
>
> If you object to software built with generative AI assistance, we respect your opinion
> and regret we cannot meet your expectations. In that case, we kindly recommend exploring
> alternative solutions. If you know of any, feel free to open an issue — we'll be happy
> to include them in our documentation.

Adds support for **GABC/NABC** (Gregorian chant notation) files to [Zed](https://zed.dev).

## Features

- **Syntax Highlighting** — Full highlighting for GABC headers, pitches, clefs, neumes, bars, alterations, style tags, NABC notation, and comments.
- **LaTeX Injection** — TeX code inside `<v>…</v>` tags and verbatim attributes (`[nv:…]`, `[gv:…]`, `[ev:…]`) gets LaTeX highlighting.
- **Bracket Matching** — Auto-matching for `()`, `[]`, `{}`, and `<>`.
- **Code Outline** — Headers appear in the document outline/symbols panel.
- **Language Server** — Diagnostics, hover, completion, and document symbols via [gregorio-lsp](https://github.com/AISCGre-BR/gregorio-lsp).
- **Quick Fixes** — Auto-fixes for many diagnostics (e.g. `bar-mixed-with-notes`, `line-break-not-at-end`, `nabc-space-in-code`) through Zed's code actions menu.
- **Document Formatting** — Format `.gabc` files via **Format Document** (`Alt+Shift+F` / `⌥⇧F`) or on save, powered by [grefmt](https://github.com/AISCGre-BR/gregorio-lsp), including the word-per-line layout for long GABC+NABC scores.

## Language server

Language server features (diagnostics, quick fixes, hover, completion,
formatting) are provided by
[gregorio-lsp](https://github.com/AISCGre-BR/gregorio-lsp). **No manual
installation is needed**: the extension resolves the server in this order:

1. `lsp.gregorio-lsp.binary.path` in Zed's `settings.json`, if set;
2. a `gregorio-lsp` executable on your `PATH`;
3. otherwise, the latest prebuilt binary for your platform is downloaded from
   the [gregorio-lsp releases](https://github.com/AISCGre-BR/gregorio-lsp/releases)
   (Linux, macOS and Windows on x86-64 and ARM64) and kept up to date
   automatically.

To use your own build instead, install it with Cargo (the `gregorio-server`
package name is required since gregorio-lsp v0.11.0):

```sh
cargo install --git https://github.com/AISCGre-BR/gregorio-lsp \
  --tag v0.13.0 gregorio-server --bin gregorio-lsp
```

or point Zed to a specific binary:

```json
{
  "lsp": {
    "gregorio-lsp": {
      "binary": { "path": "/path/to/gregorio-lsp" }
    }
  }
}
```

If you restrict `granted_extension_capabilities`, keep a `download_file`
capability for `github.com` or install the server yourself. Syntax
highlighting works without the server.

## Installation

### From Zed Extensions Marketplace

Search for "Gregorio" in **Zed → Extensions**.

### As Dev Extension

1. Clone this repository
2. In Zed, run **`zed: install dev extension`** and select the cloned directory

## Grammar

This extension uses [tree-sitter-gregorio](https://github.com/AISCGre-BR/tree-sitter-gregorio) (v0.5.2, commit `c9034de8`), a complete tree-sitter grammar for GABC+NABC notation compatible with Gregorio 6.2.0.

## Formatting

Formatting is provided by the `gregorio-lsp` language server (which embeds `grefmt`). No additional binary is required beyond the `gregorio-lsp` server already installed for diagnostics.

### Using the formatter

- **Format Document**: `Alt+Shift+F` (Linux/Windows) or `⌥⇧F` (macOS)
- **Format on save**: add to your Zed `settings.json`:
  ```json
  {
    "languages": {
      "GABC": {
        "format_on_save": "on"
      }
    }
  }
  ```

## Configuration

The extension forwards the `settings` object of `lsp.gregorio-lsp` in Zed's
`settings.json` to the server. All keys are optional; the values below are the
defaults:

```json
{
  "lsp": {
    "gregorio-lsp": {
      "settings": {
        "linting": {
          "enabled": true,
          "severity": "info",
          "onSave": false,
          "ignoreRules": []
        },
        "formatting": {
          "maxLineWidth": 80,
          "breakAfterClef": true,
          "breakAfterBar": true,
          "wordPerLine": "auto"
        }
      }
    }
  }
}
```

### Linting options

| Option | Type | Default | Description |
|---|---|---|---|
| `enabled` | boolean | `true` | Enable or disable diagnostics |
| `severity` | `"error"` \| `"warning"` \| `"info"` | `"info"` | Minimum severity shown |
| `onSave` | boolean | `false` | Lint only on save instead of on every change |
| `ignoreRules` | string[] | `[]` | Rule codes to suppress, e.g. `isolated-strophae` or `quilisma-missing-connector` (see gregorio-lsp's [CONFIG.md](https://github.com/AISCGre-BR/gregorio-lsp/blob/main/CONFIG.md) for the full list) |

### Formatting options

| Option | Type | Default | Description |
|---|---|---|---|
| `maxLineWidth` | integer | `80` | Maximum characters per notation line |
| `breakAfterClef` | boolean | `true` | Insert a blank line after each clef token |
| `breakAfterBar` | boolean | `true` | Insert a blank line after each bar token |
| `wordPerLine` | `"auto"` \| `"always"` \| `"never"` | `"auto"` | Place each word on its own line (gregorio-lsp ≥ 0.12.0). `auto` enables it when the `nabc-lines` header is 2 or greater, or when at least 25% of the words are wider than half of `maxLineWidth`. Bars and `*` / `**` stay on the line of the preceding word. |

## License

MIT — Copyright (c) 2026 AISCGre Brasil. See [LICENSE](LICENSE).