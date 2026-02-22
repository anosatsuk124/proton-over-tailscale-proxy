# Configuration Guide

This document covers all configuration options for the ProtonVPN over Tailscale Proxy system.

## Configuration Files

### Environment Variables File

**Location:** `config/.env`

The main configuration file containing sensitive credentials and system settings.

```bash
# Required Settings
PROTONVPN_USERNAME=your_protonvpn_username
PROTONVPN_PASSWORD=your_protonvpn_password
TAILSCALE_AUTH_KEY=tskey-auth-your-key-here

# Optional Settings
API_PORT=8080
FRONTEND_PORT=3000
LOG_LEVEL=info
```

**Security Note:** This file should have restrictive permissions:
```bash
chmod 600 config/.env
```

### Docker Compose Override

**Location:** `docker-compose.override.yml`

Use this for local development customizations without modifying the main compose file.

```yaml
version: '3.8'

services:
  api:
    environment:
      - RUST_LOG=debug
    volumes:
      - ./rust-backend/src:/app/src
  
  frontend:
    environment:
      - VITE_API_URL=http://localhost:8080
```

## Environment Variables Reference

### Required Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `PROTONVPN_USERNAME` | Your ProtonVPN account username | `john.doe@example.com` |
| `PROTONVPN_PASSWORD` | Your ProtonVPN account password | `your_secure_password` |
| `TAILSCALE_AUTH_KEY` | Tailscale authentication key | `tskey-auth-k123456CNTRL-...` |

### VPN Settings

| Variable | Description | Default | Options |
|----------|-------------|---------|---------|
| `VPN_SERVER` | Preferred ProtonVPN server | `US-FREE#1` | See [Server List](#protonvpn-servers) |
| `VPN_PROTOCOL` | VPN protocol to use | `wireguard` | `wireguard`, `openvpn` |
| `VPN_TIER` | ProtonVPN plan tier | `free` | `free`, `plus`, `visionary` |
| `VPN_DNS` | Custom DNS servers | `1.1.1.1,8.8.8.8` | Comma-separated IPs |

### Tailscale Settings

| Variable | Description | Default |
|----------|-------------|---------|
| `TAILSCALE_HOSTNAME` | Hostname in Tailscale network | `proton-vpn-exit` |
| `TAILSCALE_ADVERTISE_EXIT_NODE` | Advertise as exit node | `true` |
| `TAILSCALE_ADVERTISE_ROUTES` | Additional routes to advertise | `10.0.0.0/8` |
| `TAILSCALE_ACCEPT_DNS` | Use Tailscale DNS | `true` |
| `TAILSCALE_ACCEPT_ROUTES` | Accept routes from other nodes | `true` |

### API Server Settings

| Variable | Description | Default |
|----------|-------------|---------|
| `API_PORT` | HTTP API server port | `8080` |
| `API_HOST` | Bind address | `0.0.0.0` |
| `API_WORKERS` | Number of worker threads | `4` |
| `API_TIMEOUT` | Request timeout (seconds) | `30` |
| `API_RATE_LIMIT` | Requests per minute | `100` |
| `API_CORS_ORIGINS` | Allowed CORS origins | `*` |

### Frontend Settings

| Variable | Description | Default |
|----------|-------------|---------|
| `FRONTEND_PORT` | Frontend dev server port | `3000` |
| `FRONTEND_HOST` | Bind address | `0.0.0.0` |
| `VITE_API_URL` | API URL for frontend | `http://localhost:8080` |
| `VITE_WS_URL` | WebSocket URL | `ws://localhost:8080` |

### Logging Settings

| Variable | Description | Default | Options |
|----------|-------------|---------|---------|
| `LOG_LEVEL` | Logging verbosity | `info` | `trace`, `debug`, `info`, `warn`, `error` |
| `LOG_FORMAT` | Log output format | `json` | `json`, `pretty` |
| `LOG_OUTPUT` | Log destination | `stdout` | `stdout`, `file`, `both` |
| `LOG_FILE_PATH` | Path to log file | `/var/log/proton-vpn.log` | Any valid path |
| `LOG_MAX_SIZE` | Max log file size | `100MB` | Size in MB/GB |
| `LOG_MAX_FILES` | Number of rotated log files | `5` | Integer |

### Docker Settings

| Variable | Description | Default |
|----------|-------------|---------|
| `DOCKER_NETWORK` | Docker network name | `proton-vpn-net` |
| `DOCKER_SUBNET` | Network subnet | `172.20.0.0/16` |
| `DOCKER_DNS` | DNS servers for containers | `1.1.1.1,8.8.8.8` |
| `DOCKER_RESTART_POLICY` | Container restart policy | `unless-stopped` |

## ProtonVPN Server Configuration

### Server Selection

You can specify servers using the following formats:

**By Country Code:**
```bash
VPN_SERVER=US  # Any US server
VPN_SERVER=JP  # Any Japan server
```

**By Specific Server:**
```bash
VPN_SERVER=US-FREE#1  # Specific free server
VPN_SERVER=US-NL#10   # Specific Plus server
```

**By Features:**
```bash
VPN_SERVER=P2P        # Any P2P-enabled server
VPN_SERVER=TOR        # Any Tor-over-VPN server
VPN_SERVER=SECURE_CORE  # Any Secure Core server
```

### ProtonVPN Servers

**Free Tier (Limited):**
- `US-FREE#1` through `US-FREE#3`
- `NL-FREE#1` through `NL-FREE#3`
- `JP-FREE#1` through `JP-FREE#3`

**Plus Tier (Full Access):**
- All countries: `US`, `CA`, `GB`, `DE`, `NL`, `CH`, `SE`, `SG`, `JP`, `AU`, etc.
- P2P servers: `US-NL#10`, `NL-NL#15`, `CH-NL#8`
- Tor servers: `US-TX#1`, `CH-SE#1`
- Secure Core: `US-SECURE-CORE`, `CH-SECURE-CORE`

## Tailscale Configuration

### Auth Key Types

**Reusable Key (Recommended for servers):**
```bash
TAILSCALE_AUTH_KEY=tskey-auth-k123456CNTRL-abc123def456
```

Generate at: https://login.tailscale.com/admin/settings/keys

**Options:**
- Ephemeral: `false` (persist node identity)
- Reusable: `true` (can be used multiple times)
- Pre-authorized: `true` (skip approval)
- Tags: `tag:exit-node`

### Exit Node Configuration

To use this server as a Tailscale exit node:

1. **Enable in container:**
```bash
TAILSCALE_ADVERTISE_EXIT_NODE=true
```

2. **Approve in Tailscale Admin Console:**
   - Go to https://login.tailscale.com/admin/machines
   - Find your node
   - Click "Edit route settings"
   - Enable "Use as exit node"

3. **Connect from client:**
```bash
# Linux/macOS
tailscale up --exit-node=proton-vpn-exit

# Windows
tailscale up --exit-node=proton-vpn-exit
```

### Subnet Router

To route specific subnets through this node:

```bash
TAILSCALE_ADVERTISE_ROUTES=10.0.0.0/8,172.16.0.0/12,192.168.0.0/16
```

## Advanced Configuration

### WireGuard Settings

```bash
# WireGuard interface name
WG_INTERFACE=wg0

# WireGuard listen port
WG_PORT=51820

# WireGuard MTU
WG_MTU=1420

# Persistent keepalive (seconds)
WG_PERSISTENT_KEEPALIVE=25

# DNS leak protection
WG_DNS_LEAK_PROTECTION=true
```

### Firewall Configuration

The system uses iptables for traffic management. Default rules:

```bash
# Kill switch (block traffic if VPN disconnects)
ENABLE_KILL_SWITCH=true

# Allow LAN access
ALLOW_LAN=true
LAN_SUBNETS=192.168.0.0/16,10.0.0.0/8,172.16.0.0/12

# IPv6 support
ENABLE_IPV6=false
```

### Performance Tuning

```bash
# TCP optimization
TCP_CONGESTION_CONTROL=bbr
TCP_FAST_OPEN=true

# UDP buffer sizes
NET_CORE_RMEM_MAX=134217728
NET_CORE_WMEM_MAX=134217728

# Connection tracking
NET_NETFILTER_NF_CONNTRACK_MAX=2000000
```

## Configuration Examples

### Basic Setup

```bash
# Minimal configuration
PROTONVPN_USERNAME=user@example.com
PROTONVPN_PASSWORD=pass123
TAILSCALE_AUTH_KEY=tskey-auth-xxx
```

### Production Setup

```bash
# Production with all security features
PROTONVPN_USERNAME=user@example.com
PROTONVPN_PASSWORD=secure_password
TAILSCALE_AUTH_KEY=tskey-auth-xxx

VPN_SERVER=CH-NL#8
VPN_PROTOCOL=wireguard
VPN_TIER=plus
ENABLE_KILL_SWITCH=true

API_PORT=8080
API_RATE_LIMIT=60
LOG_LEVEL=warn
LOG_FORMAT=json

TAILSCALE_HOSTNAME=secure-exit-node
TAILSCALE_ADVERTISE_EXIT_NODE=true
```

### Development Setup

```bash
# Development with debugging
PROTONVPN_USERNAME=user@example.com
PROTONVPN_PASSWORD=pass123
TAILSCALE_AUTH_KEY=tskey-auth-xxx

VPN_SERVER=US-FREE#1
LOG_LEVEL=debug
LOG_FORMAT=pretty

API_PORT=8080
FRONTEND_PORT=3000
```

## Troubleshooting

### Common Configuration Issues

**Issue:** VPN container fails to start
```
Error: ProtonVPN authentication failed
```
**Solution:**
- Verify credentials in `.env`
- Check if account is active
- Ensure correct VPN tier (free/plus)

**Issue:** Tailscale authentication fails
```
Error: invalid auth key
```
**Solution:**
- Generate new auth key from Tailscale console
- Ensure key is not expired
- Check if node limit reached

**Issue:** API returns 500 errors
```
Error: Docker connection refused
```
**Solution:**
- Ensure Docker daemon is running
- Check Docker socket permissions
- Verify user is in docker group

**Issue:** Frontend can't connect to API
```
Error: Network Error
```
**Solution:**
- Check `VITE_API_URL` in frontend config
- Verify API is running on correct port
- Check firewall rules

### Validation Commands

```bash
# Validate environment file
source config/.env && echo "Config loaded successfully"

# Test Docker connectivity
docker ps

# Test ProtonVPN credentials
# (Replace with your actual check command)

# Test Tailscale auth key
curl -s "https://login.tailscale.com/xxx" # API endpoint

# Check network connectivity
ping -c 4 1.1.1.1
```

### Configuration Reload

Some settings require container restart:

```bash
# Reload configuration
docker compose down
docker compose up -d

# Or specific service
docker compose restart vpn
```

### Debugging Configuration

Enable debug logging:

```bash
# Temporarily enable debug
RUST_LOG=debug docker compose up api

# Or set in .env
LOG_LEVEL=debug
```

View effective configuration:

```bash
# Docker Compose config
docker compose config

# Environment variables
env | grep -E "PROTON|TAILSCALE|API|VPN"

# Container environment
docker exec proton-vpn env
```

## Security Best Practices

### 1. Credential Management

- Use Docker secrets for production
- Rotate auth keys regularly
- Never commit credentials to git
- Use strong, unique passwords

### 2. Network Security

- Use firewall rules to restrict API access
- Enable kill switch to prevent leaks
- Use HTTPS in production
- Limit CORS origins

### 3. Access Control

- Restrict Tailscale ACLs
- Use tags for node management
- Enable audit logging
- Implement API authentication

### 4. Regular Updates

- Keep Docker images updated
- Update Tailscale client regularly
- Monitor security advisories
- Test updates in staging

## Migration Guide

### From File-based to Docker Secrets

1. **Create secrets:**
```bash
echo "your_password" | docker secret create proton_password -
echo "your_auth_key" | docker secret create tailscale_auth -
```

2. **Update docker-compose.yml:**
```yaml
services:
  vpn:
    secrets:
      - proton_password
      - tailscale_auth
    environment:
      - PROTONVPN_PASSWORD_FILE=/run/secrets/proton_password
```

### Environment Variable Changes

When upgrading between versions, check for:
- Renamed variables
- Deprecated options
- New required settings
- Changed defaults

Always review the changelog before upgrading.
