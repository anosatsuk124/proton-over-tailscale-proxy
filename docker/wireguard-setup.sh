#!/bin/bash
set -e

# WireGuard Setup Script
# Handles WireGuard interface creation and configuration
# This is called by entrypoint.sh but can also be run standalone for debugging

WG_INTERFACE="${WG_INTERFACE:-wg0}"
WG_CONFIG="${WG_CONFIG:-/etc/wireguard/wg0.conf}"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

# Check if running as root
check_root() {
    if [[ $EUID -ne 0 ]]; then
        echo "This script must be run as root" >&2
        exit 1
    fi
}

# Load kernel module
load_module() {
    log "Loading WireGuard kernel module..."
    modprobe wireguard 2>/dev/null || {
        log "Warning: Could not load WireGuard kernel module, using userspace implementation"
    }
}

# Create WireGuard interface
create_interface() {
    log "Creating WireGuard interface ${WG_INTERFACE}..."
    
    # Remove existing interface if present
    ip link del "${WG_INTERFACE}" 2>/dev/null || true
    
    # Create new interface
    ip link add "${WG_INTERFACE}" type wireguard
}

# Configure interface with WireGuard config
configure_interface() {
    log "Configuring WireGuard interface..."
    
    if [[ ! -f "$WG_CONFIG" ]]; then
        echo "Error: WireGuard config not found at $WG_CONFIG" >&2
        exit 1
    fi
    
    # Apply WireGuard configuration
    wg setconf "${WG_INTERFACE}" "${WG_CONFIG}"
    
    # Bring up interface
    ip link set up "${WG_INTERFACE}"
    
    log "WireGuard interface configured successfully"
}

# Display interface status
show_status() {
    log "WireGuard interface status:"
    wg show "${WG_INTERFACE}" || echo "Interface not found"
    
    log "IP addresses:"
    ip addr show "${WG_INTERFACE}" 2>/dev/null || echo "Interface not found"
    
    log "Routing table:"
    ip route show dev "${WG_INTERFACE}" 2>/dev/null || echo "No routes found"
}

# Main setup function
setup() {
    check_root
    load_module
    create_interface
    configure_interface
    show_status
    log "WireGuard setup complete"
}

# Teardown function
teardown() {
    log "Tearing down WireGuard interface..."
    ip link del "${WG_INTERFACE}" 2>/dev/null || true
    log "WireGuard teardown complete"
}

# Main
case "${1:-setup}" in
    setup)
        setup
        ;;
    teardown)
        teardown
        ;;
    status)
        show_status
        ;;
    *)
        echo "Usage: $0 {setup|teardown|status}"
        exit 1
        ;;
esac
