# Codex Project Instructions

This project uses OpenSpec for spec-driven development.

For non-trivial feature work, behavioral changes, architecture changes, or refactors:

1. Check existing OpenSpec state with `openspec list` and `openspec list --specs`.
2. Create or update an OpenSpec change before implementation.
3. Keep the change artifacts under `openspec/changes/<change-name>/` aligned with the code.
4. Implement tasks from `tasks.md` and mark completed items as done.
5. Validate with `openspec validate --all` before considering the change complete.

Small mechanical fixes, formatting, dependency lockfile refreshes, and documentation-only edits can be done directly when an OpenSpec change would add no useful clarity.

Use the Rust workspace conventions documented in `README.md`.
