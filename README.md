# Implementation of the Massively-Parallel Vector Graphics paper

This repository contains an implementation of the shortcut tree division from
the paper "Massively-Parallel Vector Graphics". It consists of two parts,
a Rust program and a website, which must be run in sequence.

## Rust program

The rust program can be run with `cargo`:

```bash
cargo run -- filename.svg
```

The output will be placed in the `web/public` folder.

## Website

The website can be built and hosted with `npm`:

```bash
cd web
npm install
npm run dev
```