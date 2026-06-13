# Agent Instructions

This project uses OpenSpec for spec-driven development.

For planned changes, non-trivial feature work, behavioral changes, architecture
changes, or refactors:

1. Check existing OpenSpec state with `openspec list` and `openspec list --specs`.
2. Start with explore mode when thinking through the change.
3. Create or update the OpenSpec proposal, design, tasks, and spec files before
   implementation where appropriate.
4. Make a git commit before applying the OpenSpec change so the pre-change state
   is preserved.
5. Implement tasks from `tasks.md`, keep change artifacts under
   `openspec/changes/<change-name>/` aligned with the code, and mark completed
   items as done.
6. Make small git commits after each coherent chunk of implementation work.
7. Make a git commit after updating tests, specs, or docs for the change.
8. Run relevant tests and checks before committing where practical.
9. Validate with `openspec validate --all` before considering the change
   complete.
10. After the change is validated and archived with OpenSpec, make a final git
    commit for the archived change.

Keep commits focused and give them clear messages. Do not mix unrelated changes
in the same commit.

If the working tree already has user changes, do not overwrite or discard them.
Work around unrelated changes, and ask before touching changes whose intent is
unclear.

Small mechanical fixes, formatting, dependency lockfile refreshes, and
documentation-only edits can be done directly when an OpenSpec change would add
no useful clarity.

Use the Rust workspace conventions documented in `README.md`.
