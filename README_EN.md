# ProtonVPN Tailscale Exit Node

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![React](https://img.shields.io/badge/React-18+-61DAFB.svg)](https://reactjs.org/)

A system that provides ProtonVPN as an exit node within your Tailscale network. Deploy a secure VPN exit node that can be easily accessed by all your Tailscale clients without requiring SOCKS proxy configuration.

## Overview

This project combines ProtonVPN and Tailscale within Docker containers to provide a VPN exit node accessible by devices in your Tailscale network. The system includes a web dashboard for managing connections, and routes all network traffic through the VPN via Tailscale's exit node functionality.

### Key Features

- **ProtonVPN Integration**: Secure VPN connection using WireGuard protocol
- **Tailscale Exit Node**: Functions as an official exit node within your Tailscale network
- **All-Device Support**: No SOCKS proxy configuration needed on client side
- **Web Dashboard**: Real-time monitoring and control of connections
- **REST API**: Programmatic VPN control
- **Docker Support**: Easy deployment and scaling
- **Automatic NAT/Masquerading**: Automatic client traffic translation

## Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Tailscale      │     │  Tailscale      │     │  Tailscale      │
│  Client #1      │     │  Client #2      │     │  Client #3      │
│  (iOS/Android)  │     │  (Windows)      │     │  (Linux/macOS)  │
└────────┬────────┘     └────────┬────────┘     └────────┬────────┘
         │                       │                       │
         │    Tailscale Mesh     │    VPN (WireGuard)    │
         │         Network       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────▼─────────────┐
                    │    Tailscale Network      │
                    │    (Encrypted Mesh)       │
                    └─────────────┬─────────────┘
                                  │
                    ┌─────────────▼─────────────┐
                    │   Exit Node Container     │
                    │  ┌─────────────────────┐  │
                    │  │   tailscaled        │  │
                    │  │  (Userspace Mode)   │  │
                    │  └──────────┬──────────┘  │
                    │             │             │
                    │  ┌──────────▼──────────┐  │
                    │  │   WireGuard         │  │
                    │  │   (ProtonVPN)       │  │
                    │  └──────────┬──────────┘  │
                    │             │             │
                    │  ┌──────────▼──────────┐  │
                    │  │   NAT/Masquerade    │  │
                    │  │   (iptables)        │  │
                    │  └─────────────────────┘  │
                    └───────────────────────────┘
                                 │
                                 ▼
                    ┌───────────────────────────┐
                    │      ProtonVPN            │
                    │      (Internet)           │
                    └───────────────────────────┘

Traffic Flow:
Client -> Tailscale -> tailscaled -> WireGuard -> ProtonVPN -> Internet
```

## Quick Start

### Prerequisites

- Docker 24.0+
- Docker Compose 2.0+
- Rust 1.75+ (for development)
- Node.js 18+ (for development)
- Tailscale account
- ProtonVPN account

### Installation

1. Clone the repository:
```bash
git clone https://github.com/anosatsuk124/proton-over-tailscale-proxy.git
cd proton-over-tailscale-proxy
```

2. Configure environment variables:
```bash
cp .env.example .env
# Edit .env with your credentials
```

3. Run with Docker:
```bash
docker compose up -d
```

4. Access the web dashboard:
```
http://localhost:3000
```

## Tailscale Exit Node Configuration

### 1. Approve in Tailscale Admin Console

After starting the container, you must approve it as an exit node in the Tailscale admin console:

1. Go to [Tailscale Admin Console](https://login.tailscale.com/admin/machines)
2. Find your deployed machine (using the `TAILSCALE_HOSTNAME` you configured)
3. Click the "**...**" (menu) next to the machine name
4. Select "**Edit route settings...**"
5. Enable "**Use as exit node**"
6. Click "**Save**"

### 2. Connect from Clients

Once approved, any device in your Tailscale network can use this exit node:

**Command Line:**
```bash
# Linux/macOS
tailscale up --exit-node=proton-vpn-exit

# To stop using exit node
tailscale up --exit-node=
```

**GUI Apps:**
- **iOS/Android**: Tailscale app → Exit Node → Select your machine
- **Windows/macOS**: Menu bar/System tray → Tailscale → Exit Node → Select your machine

**Verify:**
```bash
# Check if you're routing through the exit node
curl https://ipinfo.io
```

## Configuration

### Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `PROTONVPN_USERNAME` | ProtonVPN username | ✅ |
| `PROTONVPN_PASSWORD` | ProtonVPN password | ✅ |
| `TAILSCALE_AUTH_KEY` | Tailscale authentication key | ✅ |
| `TAILSCALE_HOSTNAME` | Hostname in Tailscale network | Default: proton-vpn-exit |
| `TAILSCALE_ADVERTISE_EXIT_NODE` | Advertise as exit node | Default: true |
| `API_PORT` | API server port | Default: 8080 |
| `FRONTEND_PORT` | Frontend port | Default: 3000 |

For detailed configuration options, see [docs/CONFIGURATION.md](./docs/CONFIGURATION.md).

## API Examples

### Check Connection Status

```bash
curl http://localhost:8080/status
```

### Start VPN Connection

```bash
curl -X POST http://localhost:8080/connect \
  -H "Content-Type: application/json" \
  -d '{"server": "US-FREE#1", "protocol": "wireguard"}'
```

### Stop VPN Connection

```bash
curl -X POST http://localhost:8080/disconnect
```

### Check Exit Node Status

```bash
curl http://localhost:8080/exit-node
```

### Enable/Disable Exit Node

```bash
# Enable
curl -X POST http://localhost:8080/exit-node \
  -H "Content-Type: application/json" \
  -d '{"enabled": true}'

# Disable
curl -X POST http://localhost:8080/exit-node \
  -H "Content-Type: application/json" \
  -d '{"enabled": false}'
```

## Documentation

- [Architecture](./docs/ARCHITECTURE.md) - System design and technical decisions
- [Development Guide](./docs/DEVELOPMENT.md) - Development setup and contribution guidelines
- [API Documentation](./docs/API.md) - Complete API endpoint specifications
- [Deployment](./docs/DEPLOYMENT.md) - Production deployment instructions
- [Configuration](./docs/CONFIGURATION.md) - Configuration options and customization

## Troubleshooting

### Container fails to start

```bash
# Check logs
docker compose logs -f vpn

# Verify configuration
docker compose config
```

### VPN connection not establishing

1. Verify ProtonVPN credentials
2. Check port forwarding (WireGuard uses UDP 51820)
3. Review firewall rules

### Exit node not working

1. **Verify approval in Tailscale Admin Console**
   - Check that "Use as exit node" is enabled at [Admin Console](https://login.tailscale.com/admin/machines)

2. **Check client-side configuration**
   ```bash
   # Verify current exit node setting
   tailscale status
   
   # Ensure exit node is properly set
   tailscale up --exit-node=proton-vpn-exit
   ```

3. **Verify NAT/Masquerading is enabled**
   ```bash
   # Check inside container
   docker exec proton-vpn iptables -t nat -L
   
   # Look for MASQUERADE rule in POSTROUTING chain
   ```

4. **Check Tailscale is advertising as exit node**
   ```bash
   docker exec proton-vpn tailscale status
   
   # Check if output includes "offers exit node"
   ```

5. **Verify routing**
   ```bash
   # Check default route in container
   docker exec proton-vpn ip route
   
   # Verify WireGuard interface (wg0) is default route
   ```

### Clients cannot connect to exit node

1. **Verify Tailscale network connection**
   ```bash
   tailscale status
   ```

2. **Check exit node name is correct**
   ```bash
   # List available exit nodes
   tailscale exit-node list
   ```

3. **Check ACL settings**
   - Verify ACL allows access to exit node in Tailscale
   - [ACL Documentation](https://tailscale.com/kb/1018/acls)

See the troubleshooting section in [docs/CONFIGURATION.md](./docs/CONFIGURATION.md) for more details.

## Contributing

Contributions are welcome! Please see [docs/DEVELOPMENT.md](./docs/DEVELOPMENT.md) for details.

## License

MIT License - see [LICENSE](./LICENSE) file for details.

## Related Links

- [ProtonVPN](https://protonvpn.com/)
- [Tailscale](https://tailscale.com/)
- [Tailscale Exit Nodes](https://tailscale.com/kb/1103/exit-nodes)
- [WireGuard](https://www.wireguard.com/)

---

[日本語版 README](./README.md) もご利用いただけます。
