## Local agent note

- `.beads/` is gitignored intentionally per current Beads guidance.
- This repo is supposed to use the shared local Dolt server on `127.0.0.1:3308`.
- Required local config shape:
  - `.beads/config.yaml` needs `dolt.shared-server: true`, `dolt.mode: server`, and `dolt.database: LazyQMK`.
  - `.beads/metadata.json` needs `database=dolt`, `backend=dolt`, `dolt_mode=server`, and `dolt_database=LazyQMK`.
- If these files disappear, `bd` may fall back to embedded mode and fail with warnings like `no beads configuration found` or `no database selected`.
- If repair is needed, restore these files instead of running a destructive reinit.
