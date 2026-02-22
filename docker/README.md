# ProtonVPN + Tailscale Exit Node

A Docker container that combines ProtonVPN (via WireGuard) and Tailscale to create a secure exit node for your Tailscale network. This allows other Tailscale devices to route their traffic through ProtonVPN.

## Architecture

```
[Your Device] --> [Tailscale] --> [This Container] --> [ProtonVPN WireGuard] --> [Internet]
```

## Prerequisites

- Docker and Docker Compose installed
- ProtonVPN account with WireGuard support
- Tailscale account with admin access
- Linux host with kernel supporting WireGuard (5.6+) or wireguard-dkms

## Quick Start

### 1. Get ProtonVPN WireGuard Configuration

1. Log in to your ProtonVPN account at https://account.protonvpn.com
2. Go to Downloads → WireGuard configuration
3. Download the configuration file for your preferred server
4. Extract the following values:
   - `PrivateKey` (from [Interface] section)
   - `PublicKey` (from [Peer] section)
   - `Endpoint` (from [Peer] section)
   - `Address` (from [Interface] section)
   - `DNS` (from [Interface] section)

### 2. Get Tailscale Auth Key

1. Go to https://login.tailscale.com/admin/settings/keys
2. Click "Generate auth key..."
3. Select:
   - **Reusable**: Yes (recommended for containers)
   - **Ephemeral**: Yes (recommended for containers)
   - **Pre-approved**: Yes (if you want to skip manual approval)
4. Copy the generated auth key (starts with `tskey-auth-`)

### 3. Configure Environment Variables

Create a `.env` file in the docker directory:

```bash
# ProtonVPN Configuration (from your WireGuard config file)
PROTON_WG_PRIVATE_KEY=YOUR_PRIVATE_KEY_HERE
PROTON_WG_PUBLIC_KEY=YOUR_PUBLIC_KEY_HERE
PROTON_WG_ENDPOINT=nl-free-01.protonvpn.net:51820
PROTON_WG_DNS=10.8.0.1
PROTON_WG_ADDRESS=10.8.0.2/32
PROTON_WG_ALLOWED_IPS=0.0.0.0/0,::/0

# Tailscale Configuration
TAILSCALE_AUTH_KEY=tskey-auth-YOUR_KEY_HERE
TAILSCALE_HOSTNAME=proton-exit-node
TAILSCALE_ACCEPT_DNS=false
TAILSCALE_SSH=true

# Security
KILL_SWITCH=true
```

**Important**: Keep your `.env` file secure and never commit it to version control.

### 4. Build and Run

```bash
cd docker

# Build the image
docker-compose build

# Start the container
docker-compose up -d

# View logs
docker-compose logs -f
```

## Configuration Options

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `PROTON_WG_PRIVATE_KEY` | Yes | - | Your ProtonVPN WireGuard private key |
| `PROTON_WG_PUBLIC_KEY` | Yes | - | ProtonVPN server public key |
| `PROTON_WG_ENDPOINT` | No | `nl-free-01.protonvpn.net:51820` | ProtonVPN server endpoint |
| `PROTON_WG_DNS` | No | `10.8.0.1` | DNS server for WireGuard |
| `PROTON_WG_ADDRESS` | No | `10.8.0.2/32` | Your WireGuard IP address |
| `PROTON_WG_ALLOWED_IPS` | No | `0.0.0.0/0,::/0` | Routes to send through VPN |
| `TAILSCALE_AUTH_KEY` | Yes | - | Tailscale authentication key |
| `TAILSCALE_HOSTNAME` | No | `proton-exit-node` | Device name in Tailscale |
| `TAILSCALE_ACCEPT_DNS` | No | `false` | Accept DNS from Tailscale |
| `TAILSCALE_SSH` | No | `true` | Enable Tailscale SSH |
| `TAILSCALE_ADVERTISE_ROUTES` | No | - | Additional routes to advertise |
| `KILL_SWITCH` | No | `true` | Block traffic if VPN disconnects |
| `HEALTH_CHECK_URL` | No | `https://ipinfo.io` | URL for health checks |

## Using the Exit Node

### Enable Exit Node in Tailscale

Once the container is running, you need to approve it as an exit node in the Tailscale admin console:

1. Go to https://login.tailscale.com/admin/machines
2. Find your exit node (named `proton-exit-node` by default)
3. Click the "..." menu → "Edit route settings..."
4. Enable "Use as exit node"
5. Click "Save"

### Route Traffic Through Exit Node

On any device in your Tailscale network:

**Using Tailscale CLI:**
```bash
tailscale up --exit-node=proton-exit-node
```

**Using Tailscale GUI:**
1. Open Tailscale app
2. Click on your exit node
3. Select "Use as exit node"

### Verify Traffic Flow

On a device routing through the exit node:
```bash
# Check your public IP
curl https://ipinfo.io

# It should show the ProtonVPN server location, not your actual location
```

## Management Commands

```bash
# View container logs
docker-compose logs -f

# Check service status
docker-compose exec proton-tailscale-exit wg show
docker-compose exec proton-tailscale-exit tailscale status

# Restart services
docker-compose restart

# Stop container
docker-compose down

# View health status
docker-compose ps

# Execute health check manually
docker-compose exec proton-tailscale-exit /app/entrypoint.sh healthcheck
```

## Security Considerations

1. **Kill Switch**: Enabled by default. Blocks all traffic if VPN disconnects.
2. **Environment Variables**: Never commit credentials to version control.
3. **Host Network**: Container uses host networking for VPN functionality.
4. **Capabilities**: Requires `NET_ADMIN`, `SYS_MODULE`, and `NET_RAW`.

## Troubleshooting

### Container fails to start

Check logs:
```bash
docker-compose logs -f
```

Common issues:
- Missing required environment variables
- Invalid ProtonVPN credentials
- Invalid Tailscale auth key
- Kernel module not loaded (try `modprobe wireguard` on host)

### VPN connection issues

Verify WireGuard configuration:
```bash
docker-compose exec proton-tailscale-exit wg show
docker-compose exec proton-tailscale-exit ip addr show wg0
```

### Tailscale connection issues

Check Tailscale status:
```bash
docker-compose exec proton-tailscale-exit tailscale status
docker-compose exec proton-tailscale-exit tailscale netcheck
```

### Health check failures

Run manual health check:
```bash
docker-compose exec proton-tailscale-exit /app/entrypoint.sh healthcheck
```

## Building from Source

```bash
# Clone repository
git clone <repository>
cd docker

# Build image
docker build -t proton-tailscale-exit .

# Run manually
docker run -d \
  --name proton-exit \
  --cap-add NET_ADMIN \
  --cap-add SYS_MODULE \
  --cap-add NET_RAW \
  --sysctl net.ipv4.conf.all.src_valid_mark=1 \
  --sysctl net.ipv4.ip_forward=1 \
  --env-file .env \
  --network host \
  proton-tailscale-exit
```

## License

MIT License

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

## Acknowledgments

- [WireGuard](https://www.wireguard.com/) - Fast, modern, secure VPN tunnel
- [Tailscale](https://tailscale.com/) - Zero config VPN
- [ProtonVPN](https://protonvpn.com/) - Secure VPN service
