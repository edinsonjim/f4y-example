# f4y-example

A small server-rendered CRUD application built with Rust, Topcoat 0.9, Toasty, and SQLite.

## Requirements

- Rust toolchain
- Topcoat CLI 0.9.0 available as `topcoat`

## Run locally

Start the application with Topcoat's development command:

```sh
topcoat dev
```

The app uses `sqlite:f4y.db` in the project directory by default and applies embedded schema migrations at startup. Set `DATABASE_URL` to use another SQLite file:

```sh
DATABASE_URL=sqlite:families-local.db topcoat dev
```

Use `topcoat dev` to build and serve the frontend assets. Running the binary directly before the asset bundle exists may fail to resolve assets.

## Features

- Create, view, and edit families with a required name and optional summary.
- Soft-delete families and restore them from the deleted-families page.
- Use optimistic locking to detect stale edits and prevent overwriting newer changes.
- Browse active and deleted families with cursor-based pagination.
- Changes appear after navigation or a manual page refresh; the list does not push live updates to other open pages.

## Pages

- `/families`: active families
- `/families/new`: create a family
- `/families/{id}/edit`: edit a family
- `/families/deleted`: restore a deleted family

## Checks

Run the test suite and formatting check with:

```sh
cargo test
cargo fmt --check
```
