# Space Game

A Rust TUI space-game prototype organized as a monorepo workspace.

## Current Status

The current build is a minimal networked vertical slice rather than a full game. It has:

- a reusable ephemeris library for fictional solar-system objects,
- a local authoritative WebSocket server,
- a shared JSON client/server protocol,
- and a Ratatui TUI client.

The demo lets a client connect to the local server, list fictional solar-system objects, query distances from a fixed demo observer, view connection/game-time status, and exit cleanly.

Supported TUI commands include:

```text
help
objects
distance mars
distance mars --at 2097-01-02T00:00:00Z
distances
distances --limit 10
distances --sort distance
distances --at 2097-01-02T00:00:00Z
status
time
advance 10 minutes
where
where mars
where mars --at 2097-01-02T00:00:00Z
quit
```

Interactive TUI command entry supports Up/Down history browsing, editable
recalled commands, Ctrl-R reverse history search, and Tab completion for command
names, server-known object names, supported options, and local `quit`/`exit`.
Command history is saved for later TUI sessions; empty commands and local exit
commands are not saved.

## Project Layout

This repository is a Cargo workspace. Crates live under `crates/`:

- `crates/space-game` is the top-level game binary. It will integrate the other crates and provide the TUI application.
- `crates/space-game-ephemeris` is a reusable ephemeris library. Its Cargo package name uses hyphens, and Rust code imports it as `space_game_ephemeris`.
- `crates/space-game-protocol` contains shared serializable protocol types for the client/server boundary.
- `crates/space-server` runs the local authoritative WebSocket server and owns ephemeris queries.
- `crates/space-client-tui` runs the Ratatui client.

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

Run tests:

```sh
cargo test
```

Run the local server in one terminal:

```sh
cargo run -p space-server
```

Run the TUI client in another terminal:

```sh
cargo run -p space-client-tui
```

The client connects to `ws://127.0.0.1:4000/ws` by default.

Run one plain text command against the same server:

```sh
cargo run -p space-client-tui -- --plain --command objects
```

Run newline-delimited plain text commands from standard input:

```sh
printf 'status\nobjects\nquit\n' | cargo run -p space-client-tui -- --plain
```

Pass `--server ws://host:port/ws` to either mode to connect to a different
WebSocket endpoint.

The older top-level placeholder binary can still be run with:

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
