# User Management Backend (Rust)

User management API built with Axum, SQLx, and Postgres. It provides JWT-based
authentication, refresh tokens, role-based authorization, and OpenAPI docs.

## Table of Contents
- Overview
- Features
- Architecture
- Quick Start
- Configuration
- API Guide
- Database and Migrations
- Observability
- Tests
- Project Structure
- License

## Overview
This service exposes a REST API for user registration, authentication, profile
updates, and admin user management. It runs SQLx migrations on startup and
ships an OpenAPI spec and Swagger UI for quick inspection.

## Features
- JWT access and refresh tokens
- Role-based access control (user/admin)
- Postgres persistence with SQLx migrations
- OpenAPI + Swagger UI
- Validation for user input
- Structured logging via `tracing`

## Architecture
- `src/api`: HTTP routes, handlers, DTOs, middleware, OpenAPI docs
- `src/app`: Application services (business workflows)
- `src/domain`: Core domain models and rules
- `src/infra`: Database, JWT, password hashing

## Quick Start

### Local (Rust + Postgres)
1) Create environment file:
   - Linux/macOS: `cp .env.example .env`
   - PowerShell: `Copy-Item .env.example .env`
2) Update `JWT_SECRET` in `.env`.
3) Start Postgres (local or Docker).
4) Run the API:
   - `cargo run`

Health check: `GET http://localhost:8080/health` returns `ok`.

### Docker (API + Postgres)
- Full stack:
  - `docker compose up --build`
- Database only:
  - `docker compose up -d db`

## Configuration

### Loading order
Configuration is loaded (low to high precedence):
1) Defaults (see `src/config.rs`)
2) `.env` file (if present)
3) Environment variables

Environment variables use `__` as a nested separator.

### Environment Variables
| Name | Description | Example |
| --- | --- | --- |
| `DATABASE_URL` | Postgres connection string | `postgres://postgres:postgres@localhost:5432/user_management` |
| `JWT_SECRET` | Secret used to sign JWTs | `change_me` |
| `APP_HOST` | Bind address | `0.0.0.0` |
| `APP_PORT` | Bind port | `8080` |
| `RUST_LOG` | Log level | `info` |
| `ACCESS_TOKEN_MINUTES` | Access token TTL (minutes) | `15` |
| `REFRESH_TOKEN_DAYS` | Refresh token TTL (days) | `7` |
| `CORS_ALLOWED_ORIGINS` | Allowed origins (comma-separated) | `http://localhost:3000` |

## API Guide

### Authentication
- Register: `POST /auth/register`
- Login: `POST /auth/login`
- Refresh: `POST /auth/refresh`
- Logout: `POST /auth/logout`

Tokens:
- Access tokens must be sent as `Authorization: Bearer <token>`.
- Refresh tokens are exchanged for new access/refresh tokens.
- Logout is stateless; it does not revoke tokens on the server.

### Authorization
- New users are created with the `user` role.
- Admin-only endpoints require role `admin`.
- The first admin user must be promoted manually (e.g., via SQL update).

### Endpoints
Public:
- `POST /auth/register`
- `POST /auth/login`
- `POST /auth/refresh`
- `GET /health`

Authenticated:
- `POST /auth/logout`
- `GET /users/me`
- `PATCH /users/me`

Admin-only:
- `GET /users` (pagination)
- `GET /users/:id`
- `PATCH /users/:id`
- `DELETE /users/:id` (deactivate)

### Pagination
`GET /users` accepts:
- `page` (default `1`)
- `per_page` (default `20`, clamped to `1..100`)

### Validation Rules (Highlights)
- `email`: must be valid format
- `username`: 3-32 characters
- `password`: minimum 8 characters

### Error Response Format
All errors return JSON:
```json
{
  "message": "human-readable error"
}
```

### API Documentation
- Swagger UI: `http://localhost:8080/swagger`
- OpenAPI JSON: `http://localhost:8080/api-doc/openapi.json`

## Database and Migrations
- SQLx migrations live in `migrations/`.
- Migrations run automatically on application startup.

## Observability
- Logging via `tracing` + `RUST_LOG` (e.g., `debug`, `info`).
- HTTP tracing is enabled via `tower-http`.

## Tests
Integration tests require `DATABASE_URL` to be set.
```bash
cargo test
```

## Project Structure
- `src/api` - HTTP routes, handlers, DTOs, docs
- `src/app` - Application services
- `src/domain` - Core domain models
- `src/infra` - Database, JWT, password hashing
- `migrations` - SQLx migrations
- `tests` - Integration tests

## License
MIT. See `LICENSE`.
