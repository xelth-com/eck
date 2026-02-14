# Roadmap

## Phase 1 — MVP (Current)
- [x] Basic relay: register, push, pull, health
- [x] SQLite WAL storage
- [x] TTL-based packet expiration
- [ ] Rate limiting middleware (governor integrated, not yet wired)
- [ ] API key authentication
- [ ] Nginx config for 9eck.com/E/ and xelth.com/E/

## Phase 2 — Production Hardening
- [ ] Account management (free/pro plans)
- [ ] Per-account rate limits (1/min free, 10/min pro)
- [ ] Payload size enforcement per plan
- [ ] Long-polling for /E/pull (reduce polling frequency)
- [ ] TLS termination docs

## Phase 3 — Monitoring & Billing
- [ ] Metrics endpoint (/E/metrics)
- [ ] Usage tracking per API key
- [ ] Billing integration for pro plans
- [ ] Dashboard for relay status
