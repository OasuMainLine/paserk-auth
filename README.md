# PoC: Paseto/PASERK-based authentication in Rust

This repository demonstrates an implementation of [Paseto](https://paseto.io/) suitable for monolithic or microservice architectures.

> [!WARNING]
> This project uses redis for storing the paserk secret, which is insecure. **You should use a Key management system for securely storing and rotating your keys**

## Motivation

Traditional JWTs provide flexibility, but that can come at the cost of security and increased cognitive overhead. Developers must decide which algorithms to use for signing and encryption. Paseto reduces this complexity by specifying secure algorithms by design.

## Includes

- A scheduler for rotating PASERK keys.
- An authentication server powered by axum.
- A resource server that authenticates requests via the authentication paserk.json endpoint.

## Tasks

- `dev:auth` Run the authentication server.

- `start:auth:scheduler` Start the background scheduler that rotates PASERK keys.

- `diesel:auth` Run the diesel CLI inside ./backend/auth and sets the corresponding DATABASE_URL

## Dependencies

A short list of the most important dependencies used in this project and what they are used for:

- [axum](https://crates.io/crates/axum) — HTTP server framework used to build the authentication and resource servers.
- [rusty_paseto](https://crates.io/crates/rusty_paseto) — Paseto implementation with PASERK support (used for token creation/verification and key formats).
- [diesel](https://crates.io/crates/diesel) + [diesel-async](https://crates.io/crates/diesel-async) — Database ORM and async support for migrations and DB access.
- [tokio](https://crates.io/crates/tokio) — Asynchronous runtime used throughout the services.
- [redis](https://crates.io/crates/redis)— Job storage used by background jobs/scheduler.
- [ed25519-dalek](https://crates.io/crates/ed25519-dalek) — Cryptographic primitives for Ed25519 key handling and signatures.
- [argon2](https://crates.io/crates/argon2) / [zxcvbn](https://crates.io/crates/zxcvbn) — Password hashing and password strength checking.
- [apalis](https://crates.io/crates/apalis) — Background job scheduling and worker queues.
