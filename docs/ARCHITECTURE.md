# System Architecture

## Overview

ProtonVPN Tailscale Exit Node is designed as a microservices architecture with clear separation of concerns. The system provides a secure, scalable VPN exit node solution through the composition of specialized components.

## System Components

### 1. Frontend (React + TypeScript)

The user-facing web interface built with React and Vite for fast development and hot module replacement.

**Key Responsibilities:**
- Connection status visualization
- Start/stop VPN controls
- Real-time log streaming via WebSocket
- Configuration management interface
- Exit node status monitoring

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
- Exit node status management

**Technology Stack:**
- Rust 1.75+ (2024 edition)
- Axum web framework
- Tokio async runtime
- Bollard (Docker API client)
- Tower HTTP middleware

### 3. VPN Container (Alpine Linux)

The core VPN infrastructure running in a Docker container with ProtonVPN and Tailscale configured as an exit node.

**Key Responsibilities:**
- WireGuard tunnel to ProtonVPN
- Tailscale exit node functionality
- Traffic routing and NAT/masquerading
- Health monitoring

**Technology Stack:**
- Alpine Linux 3.19
- WireGuard tools
- Tailscale client (userspace networking mode)
- iptables (firewall and NAT)
- Supervisor (process management)

## Exit Node Architecture

### Traffic Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Client Layer                                 │
├─────────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐  │
│  │   Laptop     │  │    Phone     │  │      Other Devices       │  │
│  │  (Tailscale) │  │  (Tailscale) │  │      (Tailscale)         │  │
│  └──────┬───────┘  └──────┬───────┘  └───────────┬──────────────┘  │
└─────────┼────────────────┼──────────────────────┼─────────────────┘
          │                │                      │
          └────────────────┼──────────────────────┘
                           │ Tailscale WireGuard Mesh
                           │ (Encrypted Traffic)
                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     Tailscale Network Layer                          │
│                    (100.x.y.z/10.x.y.z IPs)                          │
└──────────────────────────┬──────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    Exit Node Container                               │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │                     tailscaled                                 │  │
│  │              (Userspace Networking Mode)                       │  │
│  │  ┌─────────────────────────────────────────────────────────┐   │  │
│  │  │              Tailscale Interface (ts0)                   │   │  │
│  │  │         (Handles Tailscale client traffic)               │   │  │
│  │  └─────────────────────────┬───────────────────────────────┘   │  │
│  └────────────────────────────┼───────────────────────────────────┘  │
│                               │                                       │
│  ┌────────────────────────────▼───────────────────────────────────┐  │
│  │                        Routing Layer                            │  │
│  │  ┌─────────────────────────────────────────────────────────┐   │  │
│  │  │            Kernel Routing Table                          │   │  │
│  │  │  - Tailscale routes (100.x.x.x/32)                      │   │  │
│  │  │  - Default route via WireGuard                          │   │  │
│  │  └─────────────────────────┬───────────────────────────────┘   │  │
│  └────────────────────────────┼───────────────────────────────────┘  │
│                               │                                       │
│  ┌────────────────────────────▼───────────────────────────────────┐  │
│  │                      WireGuard Layer                            │  │
│  │  ┌─────────────────────────────────────────────────────────┐   │  │
│  │  │              WireGuard Interface (wg0)                   │   │  │
│  │  │            (ProtonVPN Connection)                        │   │  │
│  │  │                                                          │   │  │
│  │  │   End-to-End Encryption:                                 │   │  │
│  │  │   Container <-> ProtonVPN Server <-> Internet            │   │  │
│  │  └─────────────────────────┬───────────────────────────────┘   │  │
│  └────────────────────────────┼───────────────────────────────────┘  │
│                               │                                       │
│  ┌────────────────────────────▼───────────────────────────────────┐  │
│  │                      NAT Layer                                  │  │
│  │  ┌─────────────────────────────────────────────────────────┐   │  │
│  │  │              iptables NAT Rules                          │   │  │
│  │  │                                                          │   │  │
│  │  │   *nat                                                   │   │  │
│  │  │   :POSTROUTING ACCEPT [0:0]                              │   │  │
│  │  │   -A POSTROUTING -o wg0 -j MASQUERADE                    │   │  │
│  │  │                                                          │   │  │
│  │  │   *filter                                                │   │  │
│  │  │   :FORWARD ACCEPT [0:0]                                  │   │  │
│  │  │   -A FORWARD -i ts0 -o wg0 -j ACCEPT                     │   │  │
│  │  │   -A FORWARD -i wg0 -o ts0 -m state --state RELATED,ESTABLISHED -j ACCEPT │  │
│  │  └─────────────────────────────────────────────────────────┘   │  │
│  └────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
                               │
                               ▼
                    ┌─────────────────────┐
                    │    ProtonVPN        │
                    │   (VPN Server)      │
                    └──────────┬──────────┘
                               │
                               ▼
                    ┌─────────────────────┐
                    │     Internet        │
                    └─────────────────────┘
```

### Traffic Flow Explanation

1. **Client to Tailscale**: Client devices connect via Tailscale mesh network (encrypted WireGuard tunnels between clients and exit node)

2. **Tailscale to Container**: Tailscale client (tailscaled) receives traffic in userspace networking mode

3. **Routing Decision**: Kernel routing table directs non-Tailscale traffic to WireGuard interface

4. **WireGuard Encryption**: Traffic enters WireGuard tunnel to ProtonVPN (end-to-end encrypted)

5. **NAT/Masquerading**: Source IP is translated to hide client identities (all traffic appears to come from exit node)

6. **ProtonVPN to Internet**: Decrypted traffic exits to internet via ProtonVPN server

## Userspace Networking Mode

### Why Userspace Mode?

Tailscale runs in userspace networking mode (`--tun=userspace-networking`) to avoid conflicts with WireGuard and simplify the container networking setup.

**Benefits:**
- No TUN device conflicts with WireGuard
- Simplified container networking
- Better compatibility with Docker
- Easier to manage multiple network interfaces

**How It Works:**
```
┌─────────────────────────────────────────┐
│         tailscaled Process              │
│  ┌─────────────────────────────────┐   │
│  │     Userspace TCP/IP Stack       │   │
│  │  (Tailscale's implementation)    │   │
│  └───────────────┬─────────────────┘   │
│                  │                      │
│  ┌───────────────▼─────────────────┐   │
│  │        SOCKS5/HTTP Proxy         │   │
│  │    (For local applications)      │   │
│  └─────────────────────────────────┘   │
└─────────────────────────────────────────┘
```

### Configuration

Userspace mode is enabled by default in the container:

```bash
# Container startup command
tailscaled --tun=userspace-networking --socks5-server=localhost:1055 --outbound-http-proxy-listen=localhost:1055
```

## NAT and Masquerading

### Why NAT is Required

When acting as an exit node, the container must translate client traffic to appear as if it's originating from the container itself. This is done through IP masquerading.

### NAT Configuration

```bash
# Enable IP forwarding
echo 1 > /proc/sys/net/ipv4/ip_forward

# Configure NAT masquerading
iptables -t nat -A POSTROUTING -o wg0 -j MASQUERADE

# Allow forwarding between interfaces
iptables -A FORWARD -i ts0 -o wg0 -j ACCEPT
iptables -A FORWARD -i wg0 -o ts0 -m state --state RELATED,ESTABLISHED -j ACCEPT
```

### Packet Flow

```
Incoming Packet from Client:
  Src: 100.x.x.x (Client Tailscale IP)
  Dst: 8.8.8.8 (Google DNS)

After NAT (POSTROUTING):
  Src: 10.2.0.2 (WireGuard container IP) ← Changed by MASQUERADE
  Dst: 8.8.8.8

Response from Internet:
  Src: 8.8.8.8
  Dst: 10.2.0.2

After Reverse NAT:
  Src: 8.8.8.8
  Dst: 100.x.x.x (Client Tailscale IP)
```

## Exit Node Advertisement

### How It Works

Tailscale advertises the exit node capability to the coordination server, which then propagates this information to all clients in the network.

```
┌──────────────────────────────────────────────────────────────┐
│                     Tailscale Control Plane                   │
│                   (coordination server)                       │
└───────────────────────────┬──────────────────────────────────┘
                            │
         ┌──────────────────┼──────────────────┐
         │                  │                  │
         ▼                  ▼                  ▼
┌──────────────┐   ┌──────────────┐   ┌──────────────┐
│ Exit Node    │   │   Client A   │   │   Client B   │
│ (this system)│   │   (Phone)    │   │   (Laptop)   │
└──────────────┘   └──────────────┘   └──────────────┘

Advertisement:
Exit Node: "I offer exit node service"
                ↓
Control Plane: "Noted, will inform clients"
                ↓
Clients: "Available exit nodes: [Exit Node]"
```

### Advertisement Requirements

1. **Container Configuration**: `--advertise-exit-node` flag on tailscaled
2. **Admin Approval**: Enable "Use as exit node" in admin console
3. **ACL Permissions**: Client must have permission to use exit nodes

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
│  │  │  │  - data     │    │  │WireGuard │  │Exit Node │ │ │  │  │
│  │  │  │  - state    │    │  │  (wg0)   │  │(userspace)│ │ │  │  │
│  │  │  └─────────────┘    │  └──────────┘  └──────────┘ │ │  │  │
│  │  │                     │         │                    │ │  │  │
│  │  │                     │  ┌──────▼──────┐             │ │  │  │
│  │  │                     │  │   NAT/      │             │ │  │  │
│  │  │                     │  │Masquerading │             │ │  │  │
│  │  │                     │  │  (iptables) │             │ │  │  │
│  │  │                     │  └─────────────┘             │ │  │  │
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
   a. Initialize Tailscale with auth key and --advertise-exit-node
   b. Configure WireGuard with ProtonVPN credentials
   c. Set up iptables rules for NAT/masquerading
   d. Enable IP forwarding
   e. Start supervisord to manage processes
   ↓
5. Admin approves exit node in Tailscale console
   ↓
6. Backend monitors container health
   ↓
7. WebSocket broadcasts status to connected clients
```

### 2. Exit Node Traffic Flow

```
1. Tailscale client connects to exit node
   ↓
2. Client routes traffic to exit node (100.x.x.x)
   ↓
3. Exit node receives traffic on ts0 interface
   ↓
4. Kernel routing forwards to wg0 (WireGuard)
   ↓
5. iptables NAT masquerades source IP
   ↓
6. Traffic flows through ProtonVPN tunnel
   ↓
7. Response follows reverse path with reverse NAT
```

### 3. Log Streaming Flow

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

### 4. Why Tailscale Exit Node Architecture?

- **Native Integration**: Uses Tailscale's built-in exit node feature
- **No Client Configuration**: No SOCKS proxy settings required on clients
- **Universal Support**: Works with all Tailscale clients (iOS, Android, Windows, macOS, Linux)
- **Automatic Routing**: Transparent traffic routing through VPN
- **Centralized Management**: Single point of control for all client traffic

### 5. Why Userspace Networking?

- **Conflict Avoidance**: Prevents TUN device conflicts with WireGuard
- **Simplified Setup**: No need for privileged TUN device access
- **Container Friendly**: Better support for Docker networking
- **Flexibility**: Can run multiple networking stacks simultaneously

## Security Considerations

### Network Security

1. **WireGuard Encryption**: End-to-end encryption between container and ProtonVPN
2. **Tailscale WireGuard**: Additional encrypted mesh layer between clients and exit node
3. **Double Encryption**: Client traffic is encrypted twice (Tailscale + ProtonVPN)
4. **Firewall Rules**: iptables prevents traffic leakage and ensures proper forwarding
5. **No Bind Mounts**: Sensitive files only in Docker volumes

### Exit Node Security

1. **Admin Approval Required**: Exit nodes must be approved in Tailscale console
2. **ACL Controls**: Fine-grained access control via Tailscale ACLs
3. **Audit Logging**: All connections are logged
4. **Certificate-Based Auth**: Tailscale uses cryptographic identity

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
3. **Connection Limits**: Tailscale and WireGuard handle thousands of concurrent connections

## Monitoring and Observability

### Metrics

- Container resource usage (CPU, memory, network)
- VPN connection uptime and throughput
- Exit node utilization (number of connected clients)
- API request latency and error rates

### Logging

- Structured JSON logs from Rust backend
- Container logs aggregated and streamed
- Audit logs for configuration changes
- iptables logging (optional, for debugging)

### Health Checks

- Container health endpoints
- VPN connectivity checks
- Exit node advertisement status
- API readiness/liveness probes

## Future Considerations

### Planned Enhancements

1. **Metrics Dashboard**: Prometheus + Grafana integration
2. **Multi-Provider Support**: NordVPN, Mullvad, etc.
3. **Automatic Failover**: Multiple VPN server support
4. **Split Tunneling**: Route only specific traffic through VPN
5. **Mobile App**: Native iOS/Android management apps
6. **Traffic Analytics**: Per-client bandwidth monitoring

### Technical Debt

1. **Database Integration**: Move from file-based to SQLite/PostgreSQL
2. **Authentication**: OAuth2/OIDC support for user management
3. **CI/CD**: Automated testing and deployment pipeline
4. **Configuration Management**: Better secret management (Vault, etc.)
