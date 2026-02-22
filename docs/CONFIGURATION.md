# Configuration Guide

This document covers all configuration options for the ProtonVPN Tailscale Exit Node system.

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
TAILSCALE_HOSTNAME=proton-vpn-exit
TAILSCALE_ADVERTISE_EXIT_NODE=true
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
  
  vpn:
    environment:
      - TAILSCALE_HOSTNAME=dev-exit-node
```

## Environment Variables Reference

### Required Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `PROTONVPN_USERNAME` | Your ProtonVPN account username | `john.doe@example.com` |
| `PROTONVPN_PASSWORD` | Your ProtonVPN account password | `your_secure_password` |
| `TAILSCALE_AUTH_KEY` | Tailscale authentication key | `tskey-auth-k123456CNTRL-...` |

### Tailscale Exit Node Settings

| Variable | Description | Default | Notes |
|----------|-------------|---------|-------|
| `TAILSCALE_HOSTNAME` | Hostname in Tailscale network | `proton-vpn-exit` | Must be unique in your network |
| `TAILSCALE_ADVERTISE_EXIT_NODE` | Advertise as exit node | `true` | Required for exit node functionality |
| `TAILSCALE_ACCEPT_DNS` | Use Tailscale DNS | `true` | Allows DNS via Tailscale |
| `TAILSCALE_ACCEPT_ROUTES` | Accept routes from other nodes | `true` | For subnet routing |
| `TAILSCALE_ADVERTISE_ROUTES` | Additional routes to advertise | - | Comma-separated CIDRs |

### VPN Settings

| Variable | Description | Default | Options |
|----------|-------------|---------|---------|
| `VPN_SERVER` | Preferred ProtonVPN server | `US-FREE#1` | See [Server List](#protonvpn-servers) |
| `VPN_PROTOCOL` | VPN protocol to use | `wireguard` | `wireguard`, `openvpn` |
| `VPN_TIER` | ProtonVPN plan tier | `free` | `free`, `plus`, `visionary` |
| `VPN_DNS` | Custom DNS servers | `1.1.1.1,8.8.8.8` | Comma-separated IPs |

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

## Tailscale Exit Node Configuration

### Auth Key Types

**Reusable Key (Recommended for servers):**
```bash
TAILSCALE_AUTH_KEY=tskey-auth-k123456CNTRL-abc123def456
```

Generate at: https://login.tailscale.com/admin/settings/keys

**Key Options for Exit Nodes:**
- Ephemeral: `false` (persist node identity across restarts)
- Reusable: `true` (can use same key for multiple deployments)
- Pre-authorized: `true` (skip initial approval step)
- Tags: `tag:exit-node` (for ACL management)

### Exit Node Approval Process

After deploying the container, you **must** approve the exit node in Tailscale Admin Console:

1. **Navigate to Admin Console**
   ```
   https://login.tailscale.com/admin/machines
   ```

2. **Find Your Node**
   - Look for the hostname you configured (`TAILSCALE_HOSTNAME`)
   - It will show as "connected" but not yet approved as exit node

3. **Enable Exit Node**
   - Click the "..." menu next to your node
   - Select "Edit route settings..."
   - Enable "Use as exit node"
   - Click "Save"

4. **Verify Approval**
   ```bash
   # Check exit node status via API
   curl http://localhost:8080/exit-node
   
   # Or check via tailscale CLI in container
   docker exec proton-vpn tailscale status
   ```

### Client Configuration

Once approved, clients can use the exit node:

**Command Line (Linux/macOS):**
```bash
# Connect using exit node
tailscale up --exit-node=proton-vpn-exit

# Verify connection
tailscale status

# Stop using exit node
tailscale up --exit-node=
```

**Windows:**
```powershell
# Using PowerShell
tailscale up --exit-node=proton-vpn-exit

# Or use Tailscale GUI:
# System tray → Tailscale → Exit Node → Select your node
```

**iOS/Android:**
1. Open Tailscale app
2. Tap "Exit Node" at bottom
3. Select your exit node from the list

**Verify Exit Node is Working:**
```bash
# Check your public IP (should show ProtonVPN server location)
curl https://ipinfo.io

# Check Tailscale connection
tailscale status
# Should show: "; proton-vpn-exit offers exit node"
```

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

## Advanced Configuration

### Userspace Networking

Tailscale runs in userspace mode by default for compatibility:

```bash
# This is set automatically in the container
TAILSCALE_FLAGS=--tun=userspace-networking --socks5-server=localhost:1055
```

**Benefits:**
- No TUN device conflicts with WireGuard
- Works in Docker without privileged mode for Tailscale
- Simpler networking setup

### NAT/Masquerading Configuration

The exit node automatically configures iptables for NAT:

```bash
# These are configured automatically in the container
# Enable IP forwarding
echo 1 > /proc/sys/net/ipv4/ip_forward

# NAT configuration
iptables -t nat -A POSTROUTING -o wg0 -j MASQUERADE
iptables -A FORWARD -i ts0 -o wg0 -j ACCEPT
iptables -A FORWARD -i wg0 -o ts0 -m state --state RELATED,ESTABLISHED -j ACCEPT
```

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

The system uses iptables for traffic management:

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
# Minimal configuration for exit node
PROTONVPN_USERNAME=user@example.com
PROTONVPN_PASSWORD=pass123
TAILSCALE_AUTH_KEY=tskey-auth-xxx
TAILSCALE_HOSTNAME=my-exit-node
TAILSCALE_ADVERTISE_EXIT_NODE=true
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

TAILSCALE_HOSTNAME=secure-exit-node
TAILSCALE_ADVERTISE_EXIT_NODE=true
TAILSCALE_ACCEPT_DNS=true

API_PORT=8080
API_RATE_LIMIT=60
LOG_LEVEL=warn
LOG_FORMAT=json
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

TAILSCALE_HOSTNAME=dev-exit-node
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

**Issue:** Exit node not working
```
Error: Clients cannot route through exit node
```
**Solution:**
1. Verify exit node is approved in Tailscale Admin Console
2. Check `TAILSCALE_ADVERTISE_EXIT_NODE=true` is set
3. Verify NAT/masquerading is enabled in container
4. Check ACL rules allow exit node usage

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

# Test exit node status
curl http://localhost:8080/exit-node

# Check Tailscale auth key
curl -s "https://login.tailscale.com/xxx" # API endpoint

# Check network connectivity
docker exec proton-vpn ping -c 4 1.1.1.1

# Verify iptables rules
docker exec proton-vpn iptables -t nat -L

# Check Tailscale status
docker exec proton-vpn tailscale status
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

# Tailscale status
docker exec proton-vpn tailscale status

# Exit node status
curl http://localhost:8080/exit-node | jq
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

### 3. Exit Node Security

- Use Tailscale ACLs to control who can use exit node
- Tag exit nodes for easier ACL management
- Monitor exit node usage via API
- Regularly audit connected clients

### 4. Access Control

- Restrict Tailscale ACLs
- Use tags for node management
- Enable audit logging
- Implement API authentication

### 5. Regular Updates

- Keep Docker images updated
- Update Tailscale client regularly
- Monitor security advisories
- Test updates in staging

## Migration Guide

### From SOCKS Proxy to Exit Node

If you're migrating from the SOCKS proxy architecture:

1. **Update Environment Variables**
   ```bash
   # Remove proxy settings
   # SOCKS_PORT=1080  # Remove this
   
   # Add exit node settings
   TAILSCALE_ADVERTISE_EXIT_NODE=true
   ```

2. **Update Client Configuration**
   - Remove SOCKS proxy settings from clients
   - Configure clients to use Tailscale exit node instead

3. **Approve Exit Node**
   - Follow the approval process in Tailscale Admin Console

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
      - TAILSCALE_AUTH_KEY_FILE=/run/secrets/tailscale_auth
```

### Environment Variable Changes

When upgrading between versions, check for:
- Renamed variables
- Deprecated options
- New required settings
- Changed defaults

Always review the changelog before upgrading.
