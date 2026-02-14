# Eck — Zero-Knowledge Blind Relay Server

## Description
Rust-based blind relay service for encrypted packet routing between eckWMS instances. Acts as an emergency "mailbox" when direct P2P connections fail. Zero-Knowledge: the server never has encryption keys, it only sees metadata.

## Architecture
```
┌─────────────────────────────────────────────────┐
│              Eck Relay Server (Rust)             │
│              9eck.com/E/  xelth.com/E/          │
├─────────────────────────────────────────────────┤
│  ┌──────────┐  ┌────────────┐  ┌─────────────┐ │
│  │  Axum    │  │ PostgreSQL │  │ Rate Limiter│ │
│  │  Router  │  │  (shared)  │  │ (governor)  │ │
│  └────┬─────┘  └─────┬──────┘  └─────────────┘ │
│       │               │                          │
│  ┌────┴──────────────┴─────────────────────────┐│
│  │  Handlers: register, push, pull, health     ││
│  └─────────────────────────────────────────────┘│
├─────────────────────────────────────────────────┤
│  PostgreSQL "eck" database (shared server)      │
│  - packets (encrypted blobs + metadata)         │
│  - registrations (instance heartbeats)          │
│  - accounts (api_key, plan, allowance)          │
└─────────────────────────────────────────────────┘
```

## Key Technologies
| Component | Technology |
|-----------|------------|
| Language | Rust (2024 edition) |
| Web Framework | Axum 0.8 (Tokio) |
| Database | PostgreSQL via sqlx |
| Serialization | Serde + Serde JSON |
| Rate Limiting | governor (Token Bucket) |
| IDs | UUID v4 |

## API Endpoints (prefix /E/)
| Method | Path | Description |
|--------|------|-------------|
| POST | /E/register | Heartbeat — instance reports IP:port |
| POST | /E/push | Store encrypted packet for another instance |
| GET | /E/pull/{id} | Retrieve pending packets (auto-delete after delivery) |
| GET | /E/health | Relay availability check |

## Deployment
- **Primary:** https://9eck.com/E/ (Nginx reverse proxy)
- **Fallback:** https://xelth.com/E/ (Nginx reverse proxy)
- **Port:** 3200 (internal, configurable via PORT env)

## Client Failover Logic
1. Try direct P2P connection
2. If fails → request to `https://9eck.com/E/`
3. If fails (timeout/5xx) → request to `https://xelth.com/E/`

## Related Projects
- **eckwmsgo** — Go backend (warehouse management)
- **eckwms-movFast** — Android PDA client (Kotlin)

## Security Model
- **Zero-Knowledge:** Server stores encrypted blobs, no decryption keys
- **Visible to relay:** target_instance_id, sender_instance_id, timestamps
- **Hidden from relay:** All business data (encrypted with AES-256-GCM by clients)
- **TTL:** Packets auto-expire and are cleaned up every 60 seconds
