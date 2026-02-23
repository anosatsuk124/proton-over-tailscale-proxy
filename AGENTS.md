# Agent Guidelines for ProtonVPN-over-Tailscale

This document provides guidelines for AI agents working in this repository.

## Project Overview

A system that provides ProtonVPN as an exit node within Tailscale network. Includes:
- **Docker container**: Alpine-based VPN + Tailscale exit node
- **Rust backend**: Axum-based API for orchestration
- **Frontend**: React + Vite + TypeScript dashboard

## Build & Test Commands

### Rust Backend (`rust-backend/`)
```bash
cd rust-backend
# Build
cargo build --release

# Run
cargo run

# Run single test
cargo test <test_name>

# Run all tests
cargo test

# Lint
cargo clippy -- -D warnings

# Format
cargo fmt --check

# Type check
cargo check
```

### Frontend (`frontend/`)
```bash
cd frontend
# Install dependencies
bun install

# Development
bun run dev

# Build
bun run build

# Preview production build
bun run preview
```

### Docker
```bash
# Build all services
docker compose build

# Run all services
docker compose up -d

# Run with rebuild
docker compose up -d --build

# View logs
docker compose logs -f

# Stop all
docker compose down
```

## Code Style Guidelines

### Rust
- **Edition**: 2024
- **Naming**: snake_case for functions/variables, PascalCase for types/traits
- **Error handling**: Use `thiserror` for error enums, `?` operator for propagation
- **Async**: Use `tokio` runtime, prefer `async fn` over callbacks
- **Comments**: All comments in English
- **Imports**: Group by std -> external -> crate, alphabetical within groups
- **Dependencies**: Prefer stable libraries with 2k+ GitHub stars
- **Error responses**: Use structured JSON with error codes

### TypeScript/React
- **Naming**: camelCase for functions/variables, PascalCase for components/types
- **Components**: Functional components with hooks
- **Types**: Explicit return types on exported functions
- **Comments**: All comments in English
- **Imports**: React/dependencies first, then local modules
- **Styling**: CSS modules or inline styles, no external UI libraries

### Documentation
- **Code comments**: English only
- **Development docs** (`docs/`): English only
- **User-facing docs** (README): Both English and Japanese versions

### Git
- Commit messages: English only
- Use present tense: "Add feature" not "Added feature"
- Be descriptive but concise

## Project Structure

```
rust-backend/
  src/
    main.rs          # Entry point
    lib.rs           # Library exports
    error.rs         # Error types
    routes/          # Axum route handlers
    services/        # Business logic
    models/          # Data models

frontend/
  src/
    App.tsx          # Main component
    components/      # React components
    hooks/           # Custom hooks
    types/           # TypeScript types

docker/
  Dockerfile         # VPN container image
  *.sh               # Setup scripts

config/              # Configuration files
data/                # Data storage
logs/                # Log files
```

## Environment Variables

Copy `.env.example` to `.env` and configure:
- `PROTON_WG_PRIVATE_KEY` / `PROTON_WG_PUBLIC_KEY` - WireGuard keys
- `TAILSCALE_AUTH_KEY` - Tailscale auth key
- `API_PORT` / `FRONTEND_PORT` - Service ports

## Key Technologies

- **Rust**: axum (17k+ stars), tokio (26k+ stars), serde (9k+ stars), bollard (2k+ stars)
- **Frontend**: React (220k+ stars), Vite (60k+ stars), TypeScript
- **Docker**: Alpine Linux, WireGuard, Tailscale

## Constraints

- Minimize dependencies (only 2k+ star libraries)
- Static frontend - all dynamic data from Rust API
- No external UI component libraries
- All comments in English
- User docs in both English and Japanese
