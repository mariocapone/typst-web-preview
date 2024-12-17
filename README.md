# Typst Web Preview

A simple `typst watch` wrapper to preview compiled typst documents in the browser.

## Features

- Browser preview
- Hot reloading
- Scroll restoration

## How to build

Clone the repository and run the following commands in the project root

```bash
npm install --prefix web && npm run build --prefix web && cargo build --release
```

## How to use

### NixOS

```bash
nix run github:mariocapone/typst-web-preview main.typ -- --host 0.0.0.0
```
