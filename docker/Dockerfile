# ProtonVPN + Tailscale Exit Node with SOCKS5/HTTP Proxy
# Alpine Linux based minimal image

FROM alpine:3.19

LABEL maintainer="ProtonVPN-over-Tailscale"
LABEL description="Container that routes traffic through ProtonVPN via WireGuard and provides SOCKS5/HTTP proxy through Tailscale"

# Install required packages
# wireguard-tools: WireGuard configuration and management (>2k stars)
# tailscale: Tailscale VPN client (>15k stars)
# iproute2, iptables: Network routing and firewall
# curl: Health checks and HTTP requests
# bash: Script execution
# supervisor: Process management (>6k stars)
# dante-server: SOCKS5 proxy server (well-established)
# tinyproxy: Lightweight HTTP proxy (well-established)
RUN apk add --no-cache \
    wireguard-tools \
    iproute2 \
    iptables \
    iptables-legacy \
    curl \
    bash \
    supervisor \
    dante-server \
    tinyproxy \
    ca-certificates \
    && rm -rf /var/cache/apk/*

# Install Tailscale (official binary)
# Tailscale has >15k GitHub stars, well-established
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
RUN mkdir -p /etc/wireguard /var/run/tailscale /etc/supervisor.d /app /var/log /var/run/tinyproxy /etc/tinyproxy

# Copy configuration and scripts
COPY entrypoint.sh /app/entrypoint.sh
COPY wireguard-setup.sh /app/wireguard-setup.sh
COPY supervisord.conf /etc/supervisord.conf
COPY config/dante.conf /etc/dante.conf
COPY config/tinyproxy.conf /etc/tinyproxy/tinyproxy.conf

# Make scripts executable
RUN chmod +x /app/entrypoint.sh /app/wireguard-setup.sh

# Expose proxy ports
# 1080 - SOCKS5 proxy
# 8080 - HTTP proxy
# 41641/udp - Tailscale
EXPOSE 1080 8080 41641/udp

# Health check - verify VPN, Tailscale, and proxies are operational
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD /app/entrypoint.sh healthcheck || exit 1

# Use supervisor to manage multiple processes
ENTRYPOINT ["/app/entrypoint.sh"]
CMD ["supervisord", "-c", "/etc/supervisord.conf"]
