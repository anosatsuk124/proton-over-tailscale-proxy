# Development Guide

This guide covers setting up the development environment, contributing guidelines, and best practices for the ProtonVPN over Tailscale Proxy project.

## Prerequisites

### Required Software

- **Rust 1.75+**: Install via [rustup](https://rustup.rs/)
- **Node.js 18+**: Install via [nvm](https://github.com/nvm-sh/nvm) or [fnm](https://github.com/Schniz/fnm)
- **Docker 24.0+**: [Install Docker](https://docs.docker.com/get-docker/)
- **Docker Compose 2.0+**: Usually included with Docker Desktop

### Optional but Recommended

- **mise**: Version manager for multiple tools
- **cargo-watch**: Auto-restart Rust server on file changes
- **cargo-edit**: Easy dependency management
- **prettier**: Code formatting for frontend

## Initial Setup

### 1. Clone Repository

```bash
git clone https://github.com/anosatsuk124/proton-over-tailscale-proxy.git
cd proton-over-tailscale-proxy
```

### 2. Configure Environment

```bash
# Copy example environment file
cp config/.env.example config/.env

# Edit with your credentials
nano config/.env
```

Required environment variables:
```bash
PROTONVPN_USERNAME=your_username
PROTONVPN_PASSWORD=your_password
TAILSCALE_AUTH_KEY=tskey-auth-...
```

### 3. Install Rust Dependencies

```bash
cd rust-backend
cargo build
```

### 4. Install Node.js Dependencies

```bash
cd ../frontend
npm install
```

## Development Workflow

### Running the Full Stack

#### Option 1: Docker Compose (Recommended for Testing)

```bash
# Build and start all services
docker compose up --build

# View logs
docker compose logs -f

# Stop all services
docker compose down
```

#### Option 2: Local Development (Hot Reload)

**Terminal 1 - Backend:**
```bash
cd rust-backend
cargo run
# Or with auto-reload:
cargo watch -x run
```

**Terminal 2 - Frontend:**
```bash
cd frontend
npm run dev
```

**Terminal 3 - VPN Container:**
```bash
cd docker
docker build -t proton-vpn .
docker run --privileged --rm \
  -e PROTONVPN_USERNAME=$PROTONVPN_USERNAME \
  -e PROTONVPN_PASSWORD=$PROTONVPN_PASSWORD \
  -e TAILSCALE_AUTH_KEY=$TAILSCALE_AUTH_KEY \
  proton-vpn
```

### Code Organization

```
proton-over-tailscale-proxy/
├── config/                 # Configuration files
│   └── .env               # Environment variables (not in git)
├── docker/                # Docker build files
│   ├── Dockerfile
│   ├── entrypoint.sh
│   └── supervisord.conf
├── docs/                  # Documentation
├── frontend/              # React + TypeScript frontend
│   ├── src/
│   │   ├── components/    # React components
│   │   ├── hooks/         # Custom React hooks
│   │   ├── types/         # TypeScript definitions
│   │   └── App.tsx
│   ├── package.json
│   └── tsconfig.json
├── rust-backend/          # Rust Axum backend
│   ├── src/
│   │   ├── error.rs       # Error handling
│   │   ├── lib.rs         # Library exports
│   │   ├── main.rs        # Application entry
│   │   ├── models.rs      # Data models
│   │   ├── routes/        # API route handlers
│   │   │   ├── connection.rs
│   │   │   ├── health.rs
│   │   │   ├── logs.rs
│   │   │   └── status.rs
│   │   └── services/      # Business logic
│   │       ├── config.rs
│   │       └── docker.rs
│   └── Cargo.toml
├── data/                  # Persistent data
├── logs/                  # Log files
└── README.md
```

## Coding Standards

### Rust

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for formatting: `cargo fmt`
- Use `clippy` for linting: `cargo clippy -- -D warnings`
- Document public APIs with `///`
- Prefer `thiserror` for error types
- Use `tracing` for logging

Example:
```rust
/// Start a VPN connection to the specified server
/// 
/// # Arguments
/// 
/// * `request` - Connection request containing server and protocol
/// 
/// # Returns
/// 
/// Returns `Ok(())` on success, or an `ApiError` on failure
pub async fn connect(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ConnectRequest>,
) -> Result<impl IntoResponse, ApiError> {
    tracing::info!("Connecting to server: {}", request.server);
    // ...
}
```

### TypeScript/React

- Use TypeScript strict mode
- Prefer functional components with hooks
- Use ESLint and Prettier for formatting
- Follow [React Best Practices](https://react.dev/learn)

Example:
```typescript
import { useState, useEffect } from 'react';

interface ConnectionStatus {
  connected: boolean;
  server?: string;
}

export function useConnectionStatus(): ConnectionStatus {
  const [status, setStatus] = useState<ConnectionStatus>({ connected: false });
  
  useEffect(() => {
    // Fetch status logic
  }, []);
  
  return status;
}
```

## Testing

### Backend Tests

```bash
cd rust-backend

# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Generate coverage
cargo tarpaulin --out Html
```

### Frontend Tests

```bash
cd frontend

# Run tests
npm test

# Run with coverage
npm test -- --coverage
```

### Integration Tests

```bash
# Start services
docker compose up -d

# Run integration tests
cd tests
./run-integration-tests.sh

# Cleanup
docker compose down
```

## Debugging

### Backend Debugging

1. **Set RUST_LOG**:
   ```bash
   RUST_LOG=debug cargo run
   ```

2. **Use tracing**:
   ```rust
   tracing::debug!("Variable value: {:?}", variable);
   ```

3. **Attach debugger** (VS Code):
   - Install "CodeLLDB" extension
   - Use provided launch configuration

### Frontend Debugging

1. **React DevTools**: Install browser extension
2. **Network Panel**: Check API calls in browser dev tools
3. **Console logging**:
   ```typescript
   console.log('[Debug]', data);
   ```

### Docker Debugging

```bash
# Enter running container
docker exec -it proton-vpn /bin/sh

# View container logs
docker logs -f proton-vpn

# Inspect container
docker inspect proton-vpn
```

## Contributing

### Pull Request Process

1. **Fork the repository** and create your branch from `main`
2. **Run tests** locally to ensure they pass
3. **Update documentation** if you're changing functionality
4. **Follow commit message conventions**:
   - `feat: add new feature`
   - `fix: resolve bug`
   - `docs: update documentation`
   - `refactor: code restructuring`
   - `test: add tests`
5. **Submit PR** with clear description of changes

### Code Review Checklist

- [ ] Code follows style guidelines
- [ ] Tests pass locally
- [ ] Documentation updated
- [ ] No breaking changes (or documented)
- [ ] Security considerations addressed
- [ ] Performance implications considered

### Commit Message Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Formatting changes
- `refactor`: Code restructuring
- `perf`: Performance improvements
- `test`: Test changes
- `chore`: Build/tooling changes

Example:
```
feat(api): add rate limiting to connect endpoint

Implement token bucket algorithm for connection requests
to prevent abuse. Default limit is 10 requests per minute.

Closes #123
```

## Troubleshooting Development Issues

### Rust Build Failures

**Issue**: `error: linking with 'cc' failed`
**Solution**: Install build dependencies
```bash
# Ubuntu/Debian
sudo apt-get install build-essential pkg-config libssl-dev

# macOS
xcode-select --install
```

### Node.js Module Issues

**Issue**: `Module not found` or version conflicts
**Solution**:
```bash
rm -rf node_modules package-lock.json
npm install
```

### Docker Permission Issues

**Issue**: Permission denied when running docker commands
**Solution**:
```bash
# Add user to docker group
sudo usermod -aG docker $USER
# Log out and back in
```

### Port Conflicts

**Issue**: Port already in use
**Solution**:
```bash
# Find process using port
sudo lsof -i :8080
# Or change port in config/.env
```

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Axum Documentation](https://docs.rs/axum/)
- [React Documentation](https://react.dev/)
- [Docker Documentation](https://docs.docker.com/)
- [Tailscale Documentation](https://tailscale.com/kb/)
- [WireGuard Documentation](https://www.wireguard.com/)

## Getting Help

- **GitHub Issues**: Bug reports and feature requests
- **Discussions**: General questions and ideas
- **Discord**: Real-time chat with community

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
