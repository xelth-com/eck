# Journal

## 2026-02-14 — Project Initialization & First Deploy
- Created Rust project with `cargo init`
- Stack: Axum 0.8 + PostgreSQL + Serde + governor
- Implemented 4 endpoints: `/E/health`, `/E/register`, `/E/push`, `/E/pull/{id}`
- Zero-Knowledge model: server stores encrypted blobs only
- Background task cleans expired packets every 60 seconds
- Installed Rust on server (antigravity, ARM64)
- Deployed to production:
  - https://9eck.com/E/ (primary)
  - https://xelth.com/E/ (fallback)
- Systemd service: `eck.service`
- Binary: 6MB, ~1.5MB RAM
- Initially used SQLite, switched to PostgreSQL (shared with eckwmsgo)
- Pull uses `DELETE...RETURNING` for atomic fetch-and-delete
- Full integration test passed: register, push, pull, TTL expiry
