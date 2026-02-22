# ProtonVPN over Tailscale Proxy

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![React](https://img.shields.io/badge/React-18+-61DAFB.svg)](https://reactjs.org/)

A proxy system for connecting to ProtonVPN through your Tailscale network. Easily deploy secure VPN exit nodes within your Tailscale mesh.

## Overview

This project combines ProtonVPN and Tailscale within Docker containers to provide secure VPN exit nodes accessible through your Tailscale network. The system includes a web dashboard for managing connections and monitoring status.

### Key Features

- **ProtonVPN Integration**: Secure VPN connection using WireGuard protocol
- **Tailscale Exit Node**: Functions as a VPN exit node within your Tailscale network
- **Web Dashboard**: Real-time monitoring and control of connections
- **REST API**: Programmatic VPN control
- **Docker Support**: Easy deployment and scaling

## Architecture

```
┌─────────────────┐
│   Web Browser   │
│   (React UI)    │
└────────┬────────┘
         │
         │ HTTP/WS
         │
┌────────▼────────┐
│  Rust Backend   │
│   (Axum API)    │
└────────┬────────┘
         │
         │ Docker API
         │
┌────────▼──────────────────────────┐
│        Docker Host                │
│  ┌─────────────────────────────┐ │
│  │    VPN Container            │ │
│  │  ┌──────────┐  ┌──────────┐ │ │
│  │  │ProtonVPN │──│Tailscale │ │ │
│  │  │ (Exit)   │  │(Exit Node)│ │ │
│  │  └──────────┘  └──────────┘ │ │
│  └─────────────────────────────┘ │
└───────────────────────────────────┘
```

## Quick Start

### Prerequisites

- Docker 24.0+
- Docker Compose 2.0+
- Rust 1.75+ (for development)
- Node.js 18+ (for development)

### Installation

1. Clone the repository:
```bash
git clone https://github.com/anosatsuk124/proton-over-tailscale-proxy.git
cd proton-over-tailscale-proxy
```

2. Configure environment variables:
```bash
cp config/.env.example config/.env
# Edit config/.env with your credentials
```

3. Run with Docker:
```bash
docker compose up -d
```

4. Access the web dashboard:
```
http://localhost:3000
```

## Configuration

### Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `PROTONVPN_USERNAME` | ProtonVPN username | ✅ |
| `PROTONVPN_PASSWORD` | ProtonVPN password | ✅ |
| `TAILSCALE_AUTH_KEY` | Tailscale authentication key | ✅ |
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

See the troubleshooting section in [docs/CONFIGURATION.md](./docs/CONFIGURATION.md) for more details.

## Contributing

Contributions are welcome! Please see [docs/DEVELOPMENT.md](./docs/DEVELOPMENT.md) for details.

## License

MIT License - see [LICENSE](./LICENSE) file for details.

## Related Links

- [ProtonVPN](https://protonvpn.com/)
- [Tailscale](https://tailscale.com/)
- [WireGuard](https://www.wireguard.com/)

---

[日本語版 README](./README.md) もご利用いただけます。
