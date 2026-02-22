# System Architecture

## Overview

ProtonVPN over Tailscale Proxy is designed as a microservices architecture with clear separation of concerns. The system provides a secure, scalable VPN solution through the composition of specialized components.

## System Components

### 1. Frontend (React + TypeScript)

The user-facing web interface built with React and Vite for fast development and hot module replacement.

**Key Responsibilities:**
- Connection status visualization
- Start/stop VPN controls
- Real-time log streaming via WebSocket
- Configuration management interface

**Technology Stack:**
- React 18+ with hooks
- TypeScript for type safety
- Vite for build tooling
- CSS-in-JS for styling

### 2. Backend API (Rust + Axum)

A high-performance REST API server built with Rust's Axum framework.

**Key Responsibilities:**
- Container lifecycle management via Docker API
- WebSocket connections for real-time updates
- Configuration persistence
- Health monitoring

**Technology Stack:**
- Rust 1.75+ (2024 edition)
- Axum web framework
- Tokio async runtime
- Bollard (Docker API client)
- Tower HTTP middleware

### 3. VPN Container (Alpine Linux)

The core VPN infrastructure running in a Docker container with ProtonVPN and Tailscale.

**Key Responsibilities:**
- WireGuard tunnel to ProtonVPN
- Tailscale exit node functionality
- Traffic routing and NAT
- Health monitoring

**Technology Stack:**
- Alpine Linux 3.19
- WireGuard tools
- Tailscale client
- iptables (firewall)
- Supervisor (process management)

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                          Client Layer                            │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐     ┌─────────────┐     ┌─────────────────────┐│
│  │ Web Browser │     │   API Key   │     │    CLI Tools        ││
│  │   (SPA)     │     │   Bearer    │     │  (curl, scripts)    ││
│  └──────┬──────┘     └─────────────┘     └─────────────────────┘│
└─────────┼───────────────────────────────────────────────────────┘
          │
          │ HTTP/HTTPS
          │
┌─────────▼───────────────────────────────────────────────────────┐
│                     API Gateway Layer                            │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                  Rust Axum Server                          │  │
│  │  ┌─────────────────────────────────────────────────────┐  │  │
│  │  │  CORS → Rate Limit → Auth → Router → Handlers       │  │  │
│  │  └─────────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────────┘  │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                           │ Docker API (Unix Socket)
                           │
┌──────────────────────────▼──────────────────────────────────────┐
│                    Container Runtime Layer                       │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                    Docker Engine                           │  │
│  │  ┌─────────────────────────────────────────────────────┐  │  │
│  │  │  ┌─────────────┐    ┌─────────────────────────────┐ │  │  │
│  │  │  │  Volumes    │    │      VPN Container          │ │  │  │
│  │  │  │  - config   │───▶│  ┌──────────┐  ┌──────────┐ │ │  │  │
│  │  │  │  - logs     │    │  │ProtonVPN │  │Tailscale │ │ │  │  │
│  │  │  │  - data     │    │  │WireGuard │  │  Client  │ │ │  │  │
│  │  │  └─────────────┘    │  └──────────┘  └──────────┘ │ │  │  │
│  │  │                     └─────────────────────────────┘ │  │  │
│  │  └─────────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
```

## Data Flow

### 1. VPN Connection Flow

```
1. Client sends POST /connect with server selection
   ↓
2. Backend validates request and loads configuration
   ↓
3. Docker API creates/starts VPN container
   ↓
4. Container startup sequence:
   a. Initialize Tailscale with auth key
   b. Configure WireGuard with ProtonVPN creds
   c. Set up routing rules and firewall
   d. Start supervisord to manage processes
   ↓
5. Backend monitors container health
   ↓
6. WebSocket broadcasts status to connected clients
```

### 2. Log Streaming Flow

```
1. Client opens WebSocket connection to /logs/stream
   ↓
2. Backend subscribes to Docker container logs
   ↓
3. Logs flow: Container stdout/stderr → Docker API → Backend → WebSocket → Client
   ↓
4. Client displays real-time log output
```

## Design Decisions

### 1. Why Rust for Backend?

- **Performance**: Low-latency API responses with minimal resource usage
- **Type Safety**: Compile-time guarantees prevent runtime errors
- **Async Support**: Native async/await with Tokio for concurrent connections
- **Memory Safety**: No garbage collection pauses, safe concurrency

### 2. Why Docker for VPN?

- **Isolation**: VPN runs in isolated environment without affecting host
- **Reproducibility**: Consistent environment across deployments
- **Security**: Container boundaries limit blast radius
- **Portability**: Works on any Docker-supported platform

### 3. Why WireGuard?

- **Performance**: Faster than OpenVPN with lower overhead
- **Simplicity**: Simple configuration, minimal attack surface
- **Modern**: Built-in kernel support in Linux 5.6+
- **Efficiency**: Better battery life on mobile devices

### 4. Why Tailscale + ProtonVPN?

- **Mesh Networking**: Tailscale provides secure mesh without manual VPN config
- **Exit Node**: Easy to route traffic through VPN without client-side changes
- **Privacy**: ProtonVPN adds an additional layer of privacy
- **Flexibility**: Can use any WireGuard-capable VPN provider

## Security Considerations

### Network Security

1. **WireGuard Encryption**: End-to-end encryption between client and ProtonVPN
2. **Tailscale WireGuard**: Additional encrypted mesh layer
3. **Firewall Rules**: iptables prevents traffic leakage
4. **No Bind Mounts**: Sensitive files only in Docker volumes

### API Security

1. **CORS Configuration**: Restricted to allowed origins
2. **Rate Limiting**: Prevents API abuse
3. **Authentication**: API key authentication (optional, configurable)
4. **HTTPS**: Production deployments should use TLS

### Container Security

1. **Rootless Execution**: Container runs as non-root where possible
2. **Capability Dropping**: Unnecessary capabilities removed
3. **Read-Only Filesystem**: Core system is read-only
4. **Security Scanning**: Regular image vulnerability scans

## Scalability

### Horizontal Scaling

The architecture supports horizontal scaling through:

1. **Stateless Backend**: API server can be replicated behind a load balancer
2. **Container Orchestration**: Kubernetes/Docker Swarm compatible
3. **Shared Storage**: Configuration stored in external volume/DB

### Vertical Scaling

Resource allocation can be adjusted:

1. **CPU/Memory Limits**: Configurable in Docker Compose
2. **Network Bandwidth**: Limited by host and VPN provider

## Monitoring and Observability

### Metrics

- Container resource usage (CPU, memory, network)
- VPN connection uptime and throughput
- API request latency and error rates

### Logging

- Structured JSON logs from Rust backend
- Container logs aggregated and streamed
- Audit logs for configuration changes

### Health Checks

- Container health endpoints
- VPN connectivity checks
- API readiness/liveness probes

## Future Considerations

### Planned Enhancements

1. **Metrics Dashboard**: Prometheus + Grafana integration
2. **Multi-Provider Support**: NordVPN, Mullvad, etc.
3. **Automatic Failover**: Multiple VPN server support
4. **Split Tunneling**: Route only specific traffic through VPN
5. **Mobile App**: Native iOS/Android management apps

### Technical Debt

1. **Database Integration**: Move from file-based to SQLite/PostgreSQL
2. **Authentication**: OAuth2/OIDC support for user management
3. **CI/CD**: Automated testing and deployment pipeline
