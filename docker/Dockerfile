# ProtonVPN + Tailscale Exit Node
# Routes all Tailscale client traffic through ProtonVPN via WireGuard
# Uses Tailscale userspace networking for container compatibility

FROM oven/bun:1.3.9

LABEL maintainer="ProtonVPN-over-Tailscale"
LABEL description="Container that routes Tailscale exit node traffic through ProtonVPN via WireGuard"

# Install required packages
# wireguard-tools: WireGuard configuration and management
# iproute2, iptables: Network routing and firewall for exit node
# curl: Health checks and HTTP requests
# ca-certificates: SSL/TLS certificates
# gnupg: For adding Tailscale repository key
RUN apt-get update && apt-get install -y --no-install-recommends \
    wireguard-tools \
    iproute2 \
    iptables \
    curl \
    wget \
    ca-certificates \
    gnupg \
    && rm -rf /var/lib/apt/lists/*

# Install Tailscale from official repository
RUN curl -fsSL https://pkgs.tailscale.com/stable/debian/bookworm.noarmor.gpg | tee /usr/share/keyrings/tailscale-archive-keyring.gpg \
    && curl -fsSL https://pkgs.tailscale.com/stable/debian/bookworm.tailscale-keyring.list | tee /etc/apt/sources.list.d/tailscale.list \
    && apt-get update \
    && apt-get install -y tailscale \
    && rm -rf /var/lib/apt/lists/*

# Enable auto-updates for Tailscale
RUN apt-get update && apt-get install -y --no-install-recommends \
    unattended-upgrades \
    cron \
    && rm -rf /var/lib/apt/lists/* \
    && echo 'APT::Periodic::Update-Package-Lists "1";' > /etc/apt/apt.conf.d/20auto-upgrades \
    && echo 'APT::Periodic::Unattended-Upgrade "1";' >> /etc/apt/apt.conf.d/20auto-upgrades \
    && echo 'Unattended-Upgrade::Allowed-Origins { "Tailscale stable archive"; };' >> /etc/apt/apt.conf.d/50unattended-upgrades

# Create necessary directories
RUN mkdir -p /etc/wireguard /var/run/tailscale /app /var/log /var/lib/tailscale

# Copy entrypoint script
COPY entrypoint.ts /app/entrypoint.ts

# Expose Tailscale port only (no proxy ports needed for exit node)
# 41641/udp - Tailscale wireguard port
EXPOSE 41641/udp

# Note: This container requires NET_ADMIN capability for VPN functionality
# Run with: --cap-add=NET_ADMIN --cap-add=SYS_MODULE --device=/dev/net/tun

# Health check - verify VPN and Tailscale are operational
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD bun run /app/entrypoint.ts healthcheck || exit 1

ENTRYPOINT ["bun", "run", "/app/entrypoint.ts"]
