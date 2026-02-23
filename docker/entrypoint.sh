#!/bin/bash
set -e

# ProtonVPN + Tailscale Exit Node Entrypoint
# Routes all Tailscale client traffic through ProtonVPN via WireGuard

# Configuration variables with defaults
# Strip any newlines from keys to prevent config file corruption
PROTON_WG_PRIVATE_KEY="${PROTON_WG_PRIVATE_KEY:-}"
PROTON_WG_PRIVATE_KEY=$(echo -n "$PROTON_WG_PRIVATE_KEY" | tr -d '\n\r')

PROTON_WG_PUBLIC_KEY="${PROTON_WG_PUBLIC_KEY:-}"
PROTON_WG_PUBLIC_KEY=$(echo -n "$PROTON_WG_PUBLIC_KEY" | tr -d '\n\r')

PROTON_WG_ENDPOINT="${PROTON_WG_ENDPOINT:-nl-free-01.protonvpn.net:51820}"
PROTON_WG_DNS="${PROTON_WG_DNS:-10.8.0.1}"
PROTON_WG_ADDRESS="${PROTON_WG_ADDRESS:-10.8.0.2/32}"
PROTON_WG_ALLOWED_IPS="${PROTON_WG_ALLOWED_IPS:-0.0.0.0/0,::/0}"

TAILSCALE_AUTH_KEY="${TAILSCALE_AUTH_KEY:-}"
TAILSCALE_AUTH_KEY=$(echo -n "$TAILSCALE_AUTH_KEY" | tr -d '\n\r')

TAILSCALE_HOSTNAME="${TAILSCALE_HOSTNAME:-proton-exit-node}"
TAILSCALE_ADVERTISE_ROUTES="${TAILSCALE_ADVERTISE_ROUTES:-}"
TAILSCALE_ACCEPT_DNS="${TAILSCALE_ACCEPT_DNS:-false}"
TAILSCALE_SSH="${TAILSCALE_SSH:-true}"
TAILSCALE_USERSPACE_NETWORKING="${TAILSCALE_USERSPACE_NETWORKING:-true}"

KILL_SWITCH="${KILL_SWITCH:-true}"
HEALTH_CHECK_URL="${HEALTH_CHECK_URL:-https://ipinfo.io}"

# Logging function
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

# Error handling function
error() {
    log "ERROR: $1" >&2
    exit 1
}

# Validate required environment variables
validate_config() {
    log "Validating configuration..."
    
    if [[ -z "$PROTON_WG_PRIVATE_KEY" ]]; then
        error "PROTON_WG_PRIVATE_KEY is required. Get it from your ProtonVPN WireGuard configuration."
    fi
    
    if [[ -z "$PROTON_WG_PUBLIC_KEY" ]]; then
        error "PROTON_WG_PUBLIC_KEY is required. Get it from your ProtonVPN WireGuard configuration."
    fi
    
    if [[ -z "$TAILSCALE_AUTH_KEY" ]]; then
        error "TAILSCALE_AUTH_KEY is required. Generate one at https://login.tailscale.com/admin/settings/keys"
    fi
    
    log "Configuration validation passed"
}

# Setup WireGuard configuration file (for reference, not used directly)
setup_wireguard() {
    log "Setting up WireGuard configuration..."
    
    # Validate key lengths (WireGuard keys are base64 encoded, should be ~44 chars)
    if [[ ${#PROTON_WG_PRIVATE_KEY} -lt 40 ]]; then
        error "PROTON_WG_PRIVATE_KEY appears to be invalid (length: ${#PROTON_WG_PRIVATE_KEY}, expected ~44 chars). Check your .env file."
    fi
    
    if [[ ${#PROTON_WG_PUBLIC_KEY} -lt 40 ]]; then
        error "PROTON_WG_PUBLIC_KEY appears to be invalid (length: ${#PROTON_WG_PUBLIC_KEY}, expected ~44 chars). Check your .env file."
    fi
    
    # Create config file line by line for reference/debugging purposes
    log "Creating WireGuard config with Address=${PROTON_WG_ADDRESS}"
    
    echo "[Interface]" > /etc/wireguard/wg0.conf
    echo "PrivateKey = ${PROTON_WG_PRIVATE_KEY}" >> /etc/wireguard/wg0.conf
    echo "Address = ${PROTON_WG_ADDRESS}" >> /etc/wireguard/wg0.conf
    echo "DNS = ${PROTON_WG_DNS}" >> /etc/wireguard/wg0.conf
    echo "" >> /etc/wireguard/wg0.conf
    echo "[Peer]" >> /etc/wireguard/wg0.conf
    echo "PublicKey = ${PROTON_WG_PUBLIC_KEY}" >> /etc/wireguard/wg0.conf
    echo "AllowedIPs = ${PROTON_WG_ALLOWED_IPS}" >> /etc/wireguard/wg0.conf
    echo "Endpoint = ${PROTON_WG_ENDPOINT}" >> /etc/wireguard/wg0.conf
    echo "PersistentKeepalive = 25" >> /etc/wireguard/wg0.conf
    
    chmod 600 /etc/wireguard/wg0.conf
    log "WireGuard configuration saved to /etc/wireguard/wg0.conf (for reference only)"
    
    # Debug: Show first few lines of config (hiding keys)
    log "Config file preview:"
    head -3 /etc/wireguard/wg0.conf | sed 's/PrivateKey = .*/PrivateKey = [HIDDEN]/' | while read line; do
        log "  $line"
    done
}

# Enable IP forwarding and configure sysctl for persistence
setup_networking() {
    log "Configuring networking for exit node functionality..."
    
    # Enable IP forwarding (immediate) - ignore errors as these may be read-only
    # These should already be set via docker-compose sysctls
    echo 1 > /proc/sys/net/ipv4/ip_forward 2>/dev/null || log "Note: /proc/sys/net/ipv4/ip_forward is read-only (set via docker sysctls)"
    echo 1 > /proc/sys/net/ipv6/conf/all/forwarding 2>/dev/null || log "Note: IPv6 forwarding sysctl is read-only"
    
    # Enable source route validation
    echo 1 > /proc/sys/net/ipv4/conf/all/src_valid_mark 2>/dev/null || log "Note: src_valid_mark sysctl is read-only"
    
    # Disable ICMP redirects
    echo 0 > /proc/sys/net/ipv4/conf/all/accept_redirects 2>/dev/null || true
    echo 0 > /proc/sys/net/ipv4/conf/all/send_redirects 2>/dev/null || true
    
    # Configure sysctl.conf for persistence across reboots
    cat >> /etc/sysctl.conf << 'EOF'

# Tailscale Exit Node Configuration
net.ipv4.ip_forward = 1
net.ipv6.conf.all.forwarding = 1
net.ipv4.conf.all.src_valid_mark = 1
net.ipv4.conf.all.accept_redirects = 0
net.ipv4.conf.all.send_redirects = 0
EOF
    
    log "Networking configured for IP forwarding"
}

# Setup NAT and masquerading for exit node traffic
setup_nat() {
    log "Setting up NAT and masquerading for exit node..."

    # Only flush POSTROUTING, preserve Docker's internal DNS NAT rules (DOCKER_OUTPUT chain)
    iptables -t nat -F POSTROUTING 2>/dev/null || true

    # Enable masquerading for WireGuard interface
    # This allows Tailscale client traffic to exit through ProtonVPN
    iptables -t nat -A POSTROUTING -o wg0 -j MASQUERADE

    # Also masquerade for tailscale0 interface if it exists
    iptables -t nat -A POSTROUTING -o tailscale0 -j MASQUERADE 2>/dev/null || true

    log "NAT/masquerading configured"
}

# Apply kill switch rules
apply_kill_switch() {
    if [[ "$KILL_SWITCH" == "true" ]]; then
        log "Applying kill switch rules..."
        
        # Flush existing filter rules (but NOT nat - Docker's internal DNS uses nat rules)
        iptables -F 2>/dev/null || true
        # Only flush user-defined nat chains, preserve Docker's DOCKER_OUTPUT/DOCKER_POSTROUTING
        iptables -t nat -F POSTROUTING 2>/dev/null || true
        
        # Default drop policy
        iptables -P INPUT DROP
        iptables -P FORWARD DROP
        iptables -P OUTPUT DROP
        
        # Allow loopback
        iptables -A INPUT -i lo -j ACCEPT
        iptables -A OUTPUT -o lo -j ACCEPT
        
        # Allow established connections
        iptables -A INPUT -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT
        iptables -A OUTPUT -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT
        
        # Allow WireGuard traffic
        iptables -A OUTPUT -p udp --dport 51820 -j ACCEPT
        
        # Allow Tailscale traffic (UDP and TCP for DERP relays)
        iptables -A OUTPUT -p udp --dport 41641 -j ACCEPT
        iptables -A OUTPUT -p udp --dport 3478 -j ACCEPT  # STUN
        iptables -A OUTPUT -p tcp --dport 443 -j ACCEPT   # HTTPS for control plane
        
        # Allow incoming Tailscale connections
        iptables -A INPUT -p udp --dport 41641 -j ACCEPT
        
        # Allow DNS
        iptables -A OUTPUT -p udp --dport 53 -j ACCEPT
        iptables -A OUTPUT -p tcp --dport 53 -j ACCEPT
        
        # Allow Docker internal DNS (required for container DNS resolution)
        iptables -A OUTPUT -d 127.0.0.11 -p udp --dport 53 -j ACCEPT
        iptables -A OUTPUT -d 127.0.0.11 -p tcp --dport 53 -j ACCEPT
        iptables -A INPUT -s 127.0.0.11 -p udp --sport 53 -j ACCEPT
        iptables -A INPUT -s 127.0.0.11 -p tcp --sport 53 -j ACCEPT
        
        # Allow ICMP (ping)
        iptables -A OUTPUT -p icmp -j ACCEPT
        iptables -A INPUT -p icmp -j ACCEPT
        
        # Allow traffic through WireGuard interface
        iptables -A INPUT -i wg0 -j ACCEPT
        iptables -A OUTPUT -o wg0 -j ACCEPT
        iptables -A FORWARD -i wg0 -j ACCEPT
        iptables -A FORWARD -o wg0 -j ACCEPT
        
        # Allow traffic through Tailscale interface
        iptables -A INPUT -i tailscale0 -j ACCEPT
        iptables -A OUTPUT -o tailscale0 -j ACCEPT
        iptables -A FORWARD -i tailscale0 -j ACCEPT
        iptables -A FORWARD -o tailscale0 -j ACCEPT

        # Allow Tailscale control plane bypass routes through eth0
        # These hosts have explicit routes through eth0 (set by activate_vpn_routing)
        iptables -A OUTPUT -o eth0 -p tcp --dport 443 -j ACCEPT
        iptables -A OUTPUT -o eth0 -p udp --dport 443 -j ACCEPT
        iptables -A INPUT -i eth0 -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT

        # Re-apply NAT after flushing
        setup_nat
        
        log "Kill switch applied"
    else
        log "Kill switch disabled"
        setup_nat
    fi
}

# Resolve a hostname to an IP address
resolve_host() {
    local host="$1"
    local ip=""

    # Check if it's already an IP address
    if echo "$host" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$'; then
        echo "$host"
        return
    fi

    # Try getent first (most reliable), prefer IPv4
    ip=$(getent ahostsv4 "$host" 2>/dev/null | awk '{print $1}' | head -1)
    if [[ -n "$ip" ]]; then
        echo "$ip"
        return
    fi

    # Fallback to nslookup
    ip=$(nslookup "$host" 2>/dev/null | awk '/^Address: / {print $2}' | head -1)
    if [[ -n "$ip" ]]; then
        echo "$ip"
        return
    fi

    # Last resort: dig
    ip=$(dig +short "$host" 2>/dev/null | grep -E '^[0-9]+\.' | head -1)
    echo "$ip"
}

# Start WireGuard (does NOT set default route - that happens after Tailscale connects)
start_wireguard() {
    log "Starting WireGuard..."

    # Create WireGuard interface
    ip link add wg0 type wireguard 2>/dev/null || true

    # Configure WireGuard using wg command directly
    # wg setconf doesn't understand Address/DNS fields, so we use wg set instead
    log "Setting WireGuard private key..."
    wg set wg0 private-key <(echo "${PROTON_WG_PRIVATE_KEY}")

    log "Adding WireGuard peer..."
    wg set wg0 peer "${PROTON_WG_PUBLIC_KEY}" allowed-ips "${PROTON_WG_ALLOWED_IPS}" endpoint "${PROTON_WG_ENDPOINT}" persistent-keepalive 25

    # Bring up interface
    ip link set up wg0

    # Add IP address
    log "Adding IP address ${PROTON_WG_ADDRESS} to wg0..."
    ip address add ${PROTON_WG_ADDRESS} dev wg0

    # Resolve ProtonVPN endpoint hostname to an actual IP for routing
    local proton_host=$(echo "$PROTON_WG_ENDPOINT" | cut -d':' -f1)
    PROTON_ENDPOINT_IP=$(resolve_host "$proton_host")

    if [[ -n "$PROTON_ENDPOINT_IP" ]]; then
        log "Resolved ProtonVPN endpoint ${proton_host} -> ${PROTON_ENDPOINT_IP}"
        log "Adding route to ${PROTON_ENDPOINT_IP} through eth0..."
        ip route add ${PROTON_ENDPOINT_IP}/32 via 172.20.0.1 dev eth0 2>/dev/null || \
            ip route replace ${PROTON_ENDPOINT_IP}/32 via 172.20.0.1 dev eth0 2>/dev/null || \
            log "WARNING: Could not add route to ${PROTON_ENDPOINT_IP}"
    else
        log "WARNING: Could not resolve ${proton_host} to an IP address"
        log "Adding hostname-based route as fallback..."
        ip route add ${proton_host} via 172.20.0.1 dev eth0 2>/dev/null || \
            ip route replace ${proton_host} via 172.20.0.1 dev eth0 2>/dev/null || \
            log "WARNING: Could not add route to ${proton_host}"
    fi

    # NOTE: Default route through wg0 is set later, after Tailscale connects.
    # This ensures Tailscale can reach its control plane via direct internet during auth.

    # Show WireGuard status for debugging
    log "WireGuard status:"
    wg show wg0 | head -5 | while read line; do
        log "  $line"
    done

    log "WireGuard started successfully (default route NOT yet changed)"
}

# Activate VPN routing: set default route through wg0 and add bypass routes for Tailscale
activate_vpn_routing() {
    log "Activating VPN routing..."

    # Add routes for Tailscale control plane through eth0 (bypass ProtonVPN)
    # This ensures Tailscale can maintain its connection after default route changes
    local ts_hosts="controlplane.tailscale.com log.tailscale.io login.tailscale.com"
    for host in $ts_hosts; do
        local ip=$(resolve_host "$host")
        if [[ -n "$ip" ]]; then
            log "Adding bypass route for ${host} (${ip}) through eth0"
            ip route add ${ip}/32 via 172.20.0.1 dev eth0 2>/dev/null || \
                ip route replace ${ip}/32 via 172.20.0.1 dev eth0 2>/dev/null || true
        else
            log "WARNING: Could not resolve ${host}"
        fi
    done

    # Also add bypass routes for Tailscale DERP servers (common ranges)
    # DERP servers use TCP 443, and their IPs are in the Tailscale control plane
    # We route all established connections through their original interface

    # Replace default route: wg0 gets priority, eth0 becomes fallback
    log "Setting default route through WireGuard..."
    # Delete the existing eth0 default route and re-add with high metric as fallback
    local eth0_gw=$(ip route | grep 'default via' | grep eth0 | awk '{print $3}')
    if [[ -n "$eth0_gw" ]]; then
        ip route del default via ${eth0_gw} dev eth0 2>/dev/null || true
        ip route add default via ${eth0_gw} dev eth0 metric 100 2>/dev/null || true
    fi
    # Add wg0 default route with low metric (highest priority)
    ip route add default dev wg0 metric 10 2>/dev/null || ip route replace default dev wg0 metric 10

    log "VPN routing activated - traffic now flows through ProtonVPN"
}

# Configure tailscale serve to expose frontend over HTTPS
setup_tailscale_serve() {
    if [[ "${TAILSCALE_SERVE_FRONTEND:-true}" == "true" ]]; then
        log "Configuring tailscale serve for frontend (HTTPS 443 -> localhost:80)..."
        tailscale serve --bg --https=443 http://localhost:80
        log "Frontend is now accessible via Tailscale HTTPS (port 443)"
    fi
}

# Stop WireGuard
stop_wireguard() {
    log "Stopping WireGuard..."
    ip link del wg0 2>/dev/null || true
}

# Start Tailscale with userspace networking
start_tailscale() {
    log "Starting Tailscale with userspace networking..."
    
    # Build tailscale up command arguments
    local TS_ARGS=""
    
    TS_ARGS="${TS_ARGS} --authkey=${TAILSCALE_AUTH_KEY}"
    TS_ARGS="${TS_ARGS} --hostname=${TAILSCALE_HOSTNAME}"
    TS_ARGS="${TS_ARGS} --advertise-exit-node"
    
    if [[ "$TAILSCALE_ACCEPT_DNS" == "true" ]]; then
        TS_ARGS="${TS_ARGS} --accept-dns=true"
    else
        TS_ARGS="${TS_ARGS} --accept-dns=false"
    fi
    
    if [[ "$TAILSCALE_SSH" == "true" ]]; then
        TS_ARGS="${TS_ARGS} --ssh"
    fi
    
    if [[ -n "$TAILSCALE_ADVERTISE_ROUTES" ]]; then
        TS_ARGS="${TS_ARGS} --advertise-routes=${TAILSCALE_ADVERTISE_ROUTES}"
    fi
    
    # Start tailscaled daemon with userspace networking for container compatibility
    # This mode doesn't require /dev/net/tun device
    if [[ "$TAILSCALE_USERSPACE_NETWORKING" == "true" ]]; then
        log "Using userspace networking mode (no TUN device required)"
        tailscaled --tun=userspace-networking \
                   --state=/var/lib/tailscale/tailscaled.state \
                   --socket=/var/run/tailscale/tailscaled.sock &
    else
        log "Using kernel TUN mode"
        tailscaled --state=/var/lib/tailscale/tailscaled.state \
                   --socket=/var/run/tailscale/tailscaled.sock &
    fi
    
    local TSD_PID=$!
    
    # Wait for daemon to start
    sleep 2
    
    # Bring up Tailscale
    tailscale up ${TS_ARGS}
    
    log "Tailscale started successfully with exit node enabled"
    log "Tailscale IP: $(tailscale ip -4 2>/dev/null || echo 'N/A')"
    
    # Display exit node status
    log "Exit node status:"
    tailscale status 2>/dev/null | head -5 || log "Status not available yet"
}

# Stop Tailscale
stop_tailscale() {
    log "Stopping Tailscale..."
    tailscale down 2>/dev/null || true
    pkill tailscaled 2>/dev/null || true
}

# Cleanup function
cleanup() {
    log "Received shutdown signal, cleaning up..."
    stop_tailscale
    stop_wireguard
    log "Cleanup complete"
    exit 0
}

# Health check function
healthcheck() {
    # Check if WireGuard interface exists and is up
    if ! ip link show wg0 >/dev/null 2>&1; then
        echo "FAIL: WireGuard interface not found"
        return 1
    fi
    
    # Check if Tailscale is running
    if ! pgrep tailscaled >/dev/null 2>&1; then
        echo "FAIL: Tailscale daemon not running"
        return 1
    fi
    
    # Check if Tailscale is connected
    if ! tailscale status --json 2>/dev/null | grep -q '"Online": *true'; then
        echo "FAIL: Tailscale not connected"
        return 1
    fi
    
    # Check internet connectivity through VPN
    if ! curl -s --max-time 10 -o /dev/null -w "%{http_code}" "$HEALTH_CHECK_URL" | grep -q "200"; then
        echo "FAIL: Cannot reach internet through VPN"
        return 1
    fi
    
    echo "OK: All services healthy"
    return 0
}

# Main function
main() {
    # Set up signal handlers
    trap cleanup SIGTERM SIGINT
    
    # Check if healthcheck mode
    if [[ "$1" == "healthcheck" ]]; then
        healthcheck
        exit $?
    fi
    
    log "Starting ProtonVPN + Tailscale Exit Node..."
    log "Traffic flow: Tailscale clients -> Tailscale daemon -> WireGuard -> ProtonVPN"
    
    # Validate and setup
    validate_config
    setup_wireguard
    setup_networking
    
    # Start services WITHOUT kill switch and WITHOUT changing default route
    # This allows Tailscale to reach its control plane via direct internet
    log "Starting services (default route stays on eth0 until Tailscale connects)..."
    start_wireguard
    start_tailscale

    # Wait for Tailscale to be ready (give it time to authenticate)
    log "Waiting for Tailscale to establish connection..."
    local attempts=0
    local max_attempts=30
    while [[ $attempts -lt $max_attempts ]]; do
        if tailscale status --json 2>/dev/null | grep -q '"Online": *true'; then
            log "Tailscale is connected!"
            break
        fi
        attempts=$((attempts + 1))
        log "Waiting for Tailscale connection... ($attempts/$max_attempts)"
        sleep 2
    done

    # Now activate VPN routing and kill switch after Tailscale is connected
    if [[ $attempts -lt $max_attempts ]]; then
        # Tailscale is connected - safe to route traffic through ProtonVPN
        activate_vpn_routing

        # Configure tailscale serve for frontend HTTPS access
        setup_tailscale_serve

        if [[ "$KILL_SWITCH" == "true" ]]; then
            log "Services are ready, applying kill switch..."
            apply_kill_switch
        fi
    else
        log "WARNING: Tailscale failed to connect within timeout"
        log "NOT activating VPN routing or kill switch to allow debugging"
        log "Check your .env configuration and network connectivity"
    fi
    
    log "All services started successfully!"
    log "This node is now advertising as an exit node on your Tailscale network"
    log "Enable it in Tailscale admin panel: https://login.tailscale.com/admin/machines"
    
    # Keep script running and monitor services
    while true; do
        # Check WireGuard
        if ! ip link show wg0 >/dev/null 2>&1; then
            log "WireGuard interface down, attempting to restart..."
            start_wireguard
        fi
        
        # Check Tailscale
        if ! pgrep tailscaled >/dev/null 2>&1; then
            log "Tailscale daemon down, attempting to restart..."
            start_tailscale
        fi
        
        sleep 30
    done
}

# Run main function
main "$@"
