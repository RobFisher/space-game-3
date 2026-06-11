# Space Game

A Rust TUI game project organized as a monorepo workspace.

## Project Layout

This repository is a Cargo workspace. Crates live under `crates/`:

- `crates/space-game` is the top-level game binary. It will integrate the other crates and provide the TUI application.
- `crates/space-game-ephemeris` is a reusable ephemeris library. Its Cargo package name uses hyphens, and Rust code imports it as `space_game_ephemeris`.

Design files and implementation notes for a crate should live alongside that crate when they are crate-specific. Shared design notes can go in a top-level `docs/` directory when needed.

## Development

Enter the Nix development shell:

```sh
nix develop path:.
```

Build the workspace:

```sh
cargo build
```

Run the top-level game binary:

```sh
cargo run -p space-game
```

## OpenSpec

This project uses [OpenSpec](https://openspec.dev/) for spec-driven development. The Nix dev shell provides the `openspec` CLI.

Useful commands:

```sh
openspec list
openspec list --specs
openspec validate --all
```

Codex is configured through project-local OpenSpec skills in `.codex/skills/`. OpenSpec also generated global Codex prompts in `/home/rob/.codex/prompts/`; this repository mirrors them in `.codex/prompts/` so Codex builds that discover project-local prompts can expose `/opsx:*` commands.
