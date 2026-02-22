# ProtonVPN + Tailscale Exit Node
# Routes all Tailscale client traffic through ProtonVPN via WireGuard
# Uses Tailscale userspace networking for container compatibility

FROM alpine:3.19

LABEL maintainer="ProtonVPN-over-Tailscale"
LABEL description="Container that routes Tailscale exit node traffic through ProtonVPN via WireGuard"

# Install required packages
# wireguard-tools: WireGuard configuration and management
# iproute2, iptables: Network routing and firewall for exit node
# curl: Health checks and HTTP requests
# bash: Script execution
# supervisor: Process management
# ca-certificates: SSL/TLS certificates
RUN apk add --no-cache \
    wireguard-tools \
    iproute2 \
    iptables \
    iptables-legacy \
    curl \
    bash \
    supervisor \
    ca-certificates \
    && rm -rf /var/cache/apk/*

# Install Tailscale (official binary)
ARG TAILSCALE_VERSION=1.58.2
RUN apk add --no-cache --virtual .build-deps \
    tar \
    && wget -q -O /tmp/tailscale.tgz \
    "https://pkgs.tailscale.com/stable/tailscale_${TAILSCALE_VERSION}_amd64.tgz" \
    && tar -xzf /tmp/tailscale.tgz -C /tmp \
    && cp /tmp/tailscale_${TAILSCALE_VERSION}_amd64/tailscale /usr/local/bin/tailscale \
    && cp /tmp/tailscale_${TAILSCALE_VERSION}_amd64/tailscaled /usr/local/bin/tailscaled \
    && chmod +x /usr/local/bin/tailscale /usr/local/bin/tailscaled \
    && rm -rf /tmp/tailscale* \
    && apk del .build-deps

# Create necessary directories
RUN mkdir -p /etc/wireguard /var/run/tailscale /etc/supervisor.d /app /var/log /var/lib/tailscale

# Copy configuration and scripts
COPY entrypoint.sh /app/entrypoint.sh
COPY wireguard-setup.sh /app/wireguard-setup.sh
COPY supervisord.conf /etc/supervisord.conf

# Make scripts executable
RUN chmod +x /app/entrypoint.sh /app/wireguard-setup.sh

# Expose Tailscale port only (no proxy ports needed for exit node)
# 41641/udp - Tailscale wireguard port
EXPOSE 41641/udp

# Note: This container requires NET_ADMIN capability for VPN functionality
# Run with: --cap-add=NET_ADMIN --cap-add=SYS_MODULE --device=/dev/net/tun

# Health check - verify VPN and Tailscale are operational
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD /app/entrypoint.sh healthcheck || exit 1

# Use supervisor to manage multiple processes
ENTRYPOINT ["/app/entrypoint.sh"]
CMD ["supervisord", "-c", "/etc/supervisord.conf"]
