# Deployment Guide

This guide covers deploying the ProtonVPN Tailscale Exit Node system to production environments.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Deployment Options](#deployment-options)
- [Docker Compose Deployment](#docker-compose-deployment)
- [Kubernetes Deployment](#kubernetes-deployment)
- [Cloud Deployment](#cloud-deployment)
- [Tailscale Exit Node Setup](#tailscale-exit-node-setup)
- [Client Configuration](#client-configuration)
- [Security Considerations](#security-considerations)
- [Monitoring](#monitoring)
- [Backup and Recovery](#backup-and-recovery)

## Prerequisites

### Required Credentials

Before deployment, ensure you have:

1. **ProtonVPN Account**: Sign up at [protonvpn.com](https://protonvpn.com)
2. **Tailscale Account**: Sign up at [tailscale.com](https://tailscale.com)
3. **Tailscale Auth Key**: Generate at [login.tailscale.com/admin/settings/keys](https://login.tailscale.com/admin/settings/keys)
   - Use "Reusable" key for server deployments
   - Enable "Pre-authorized" to skip approval step
   - Add tag `tag:exit-node` for ACL management

### System Requirements

**Minimum:**
- 2 CPU cores
- 2 GB RAM
- 10 GB disk space
- Docker 24.0+

**Recommended:**
- 4 CPU cores
- 4 GB RAM
- 20 GB SSD storage
- Docker 24.0+ with Compose plugin

### Network Requirements

- Outbound HTTPS (443) for API and updates
- Outbound UDP 51820 for WireGuard
- Outbound UDP 41641 for Tailscale
- Inbound TCP 8080 for API (configurable)
- Inbound TCP 3000 for Frontend (configurable)

## Deployment Options

### Option 1: Single Server (Recommended for Small Deployments)

Best for: Personal use, small teams (< 10 users)

**Pros:**
- Simple setup and maintenance
- Low resource overhead
- Easy backup/restore

**Cons:**
- Single point of failure
- Limited scalability

### Option 2: High Availability

Best for: Production workloads, larger teams

**Pros:**
- Fault tolerance
- Better performance
- Rolling updates

**Cons:**
- More complex setup
- Higher resource requirements

### Option 3: Cloud-Native

Best for: Enterprise deployments, multi-region

**Pros:**
- Auto-scaling
- Managed services
- Global distribution

**Cons:**
- Vendor lock-in
- Higher costs

## Docker Compose Deployment

### Production Configuration

1. **Create production directory:**
```bash
mkdir -p /opt/proton-vpn-exit-node
cd /opt/proton-vpn-exit-node
```

2. **Create `docker-compose.yml`:**

```yaml
version: '3.8'

services:
  api:
    build: 
      context: ./rust-backend
      dockerfile: Dockerfile
    container_name: proton-vpn-api
    restart: unless-stopped
    environment:
      - RUST_LOG=info
      - API_PORT=8080
    ports:
      - "8080:8080"
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
      - ./config:/app/config:ro
      - ./logs:/app/logs
    networks:
      - proton-vpn-net
    depends_on:
      - vpn
    healthcheck:
      test: ["CMD", "wget", "-q", "--spider", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

  frontend:
    build:
      context: ./frontend
      dockerfile: Dockerfile
    container_name: proton-vpn-frontend
    restart: unless-stopped
    ports:
      - "3000:3000"
    environment:
      - VITE_API_URL=http://api:8080
    networks:
      - proton-vpn-net
    depends_on:
      - api

  vpn:
    build:
      context: ./docker
      dockerfile: Dockerfile
    container_name: proton-vpn
    restart: unless-stopped
    privileged: true
    cap_add:
      - NET_ADMIN
      - SYS_MODULE
    sysctls:
      - net.ipv4.conf.all.src_valid_mark=1
      - net.ipv6.conf.all.disable_ipv6=0
      - net.ipv4.ip_forward=1
    environment:
      - PROTONVPN_USERNAME=${PROTONVPN_USERNAME}
      - PROTONVPN_PASSWORD=${PROTONVPN_PASSWORD}
      - TAILSCALE_AUTH_KEY=${TAILSCALE_AUTH_KEY}
      - TAILSCALE_HOSTNAME=${TAILSCALE_HOSTNAME:-proton-vpn-exit}
      - TAILSCALE_ADVERTISE_EXIT_NODE=true
    volumes:
      - vpn-config:/etc/wireguard
      - tailscale-state:/var/lib/tailscale
      - ./logs:/var/log
    networks:
      - proton-vpn-net
    dns:
      - 1.1.1.1
      - 8.8.8.8

volumes:
  vpn-config:
    driver: local
  tailscale-state:
    driver: local

networks:
  proton-vpn-net:
    driver: bridge
```

3. **Create environment file:**

```bash
cat > .env << 'EOF'
# ProtonVPN Credentials
PROTONVPN_USERNAME=your_protonvpn_username
PROTONVPN_PASSWORD=your_protonvpn_password

# Tailscale Authentication
TAILSCALE_AUTH_KEY=tskey-auth-your-key-here
TAILSCALE_HOSTNAME=proton-vpn-exit

# Optional Settings
API_PORT=8080
FRONTEND_PORT=3000
LOG_LEVEL=info
EOF
chmod 600 .env
```

4. **Deploy:**
```bash
docker compose up -d
```

5. **Verify deployment:**
```bash
# Check container status
docker compose ps

# Check logs
docker compose logs -f

# Test API
curl http://localhost:8080/health

# Check exit node status
curl http://localhost:8080/exit-node
```

### SSL/TLS with Traefik

For production with HTTPS:

```yaml
version: '3.8'

services:
  traefik:
    image: traefik:v2.10
    container_name: traefik
    command:
      - "--api.insecure=true"
      - "--providers.docker=true"
      - "--entrypoints.websecure.address=:443"
      - "--certificatesresolvers.letsencrypt.acme.tlschallenge=true"
      - "--certificatesresolvers.letsencrypt.acme.email=admin@yourdomain.com"
      - "--certificatesresolvers.letsencrypt.acme.storage=/letsencrypt/acme.json"
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
      - ./letsencrypt:/letsencrypt
    networks:
      - proton-vpn-net

  api:
    build: ./rust-backend
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.api.rule=Host(`api.yourdomain.com`)"
      - "traefik.http.routers.api.tls.certresolver=letsencrypt"
    # ... rest of config

  frontend:
    build: ./frontend
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.frontend.rule=Host(`vpn.yourdomain.com`)"
      - "traefik.http.routers.frontend.tls.certresolver=letsencrypt"
    # ... rest of config

  # ... vpn service remains unchanged
```

## Tailscale Exit Node Setup

After deployment, you must enable the exit node in Tailscale Admin Console:

### 1. Access Tailscale Admin Console

Navigate to: https://login.tailscale.com/admin/machines

### 2. Find Your Node

- Look for the hostname you configured (e.g., `proton-vpn-exit`)
- Status should show as "Connected"

### 3. Enable Exit Node

1. Click the **...** (three dots) menu next to your node
2. Select **"Edit route settings..."**
3. Enable **"Use as exit node"**
4. Click **"Save"**

### 4. Verify Exit Node Status

```bash
# Via API
curl http://localhost:8080/exit-node

# Expected response:
{
  "enabled": true,
  "advertised": true,
  "approved": true,
  "hostname": "proton-vpn-exit",
  "tailscale_ip": "100.x.y.z"
}
```

### 5. Configure ACLs (Optional but Recommended)

Edit your Tailscale ACL policy to control who can use the exit node:

```json
{
  "acls": [
    {
      "action": "accept",
      "src": ["autogroup:member"],
      "dst": ["*:*"]
    }
  ],
  "nodeAttrs": [
    {
      "target": ["tag:exit-node"],
      "attr": ["funnel"]
    }
  ],
  "tagOwners": {
    "tag:exit-node": ["autogroup:admin"]
  }
}
```

## Client Configuration

### macOS

```bash
# Using exit node
tailscale up --exit-node=proton-vpn-exit

# Stop using exit node
tailscale up --exit-node=

# Or use GUI: Menu bar → Tailscale → Exit Node → Select your node
```

### Linux

```bash
# Using exit node
tailscale up --exit-node=proton-vpn-exit

# Verify status
tailscale status

# Stop using exit node
tailscale up --exit-node=
```

### Windows

**PowerShell:**
```powershell
tailscale up --exit-node=proton-vpn-exit
```

**GUI:**
1. Click Tailscale icon in system tray
2. Select "Exit Node"
3. Choose your exit node from the list

### iOS

1. Open Tailscale app
2. Tap "Exit Node" at bottom of screen
3. Select your exit node from the list
4. A VPN icon will appear in status bar when active

### Android

1. Open Tailscale app
2. Tap "Exit Node" at bottom of screen
3. Select your exit node from the list
4. Connection notification will appear

### Verification

**Verify Exit Node is Working:**
```bash
# Check your public IP (should show ProtonVPN server location)
curl https://ipinfo.io

# Check Tailscale connection details
tailscale status

# Should see: "; proton-vpn-exit offers exit node"
```

## Kubernetes Deployment

### Namespace and Secrets

```yaml
# namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: proton-vpn
---
# secrets.yaml
apiVersion: v1
kind: Secret
metadata:
  name: proton-vpn-secrets
  namespace: proton-vpn
type: Opaque
stringData:
  PROTONVPN_USERNAME: "your_username"
  PROTONVPN_PASSWORD: "your_password"
  TAILSCALE_AUTH_KEY: "tskey-auth-..."
```

### VPN Deployment

```yaml
# vpn-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: proton-vpn
  namespace: proton-vpn
spec:
  replicas: 1
  selector:
    matchLabels:
      app: proton-vpn
  template:
    metadata:
      labels:
        app: proton-vpn
    spec:
      hostNetwork: true  # Required for VPN
      containers:
      - name: vpn
        image: your-registry/proton-vpn:latest
        securityContext:
          privileged: true
          capabilities:
            add:
              - NET_ADMIN
              - SYS_MODULE
        envFrom:
        - secretRef:
            name: proton-vpn-secrets
        env:
        - name: TAILSCALE_HOSTNAME
          value: "k8s-exit-node"
        - name: TAILSCALE_ADVERTISE_EXIT_NODE
          value: "true"
        volumeMounts:
        - name: wireguard-config
          mountPath: /etc/wireguard
        - name: tailscale-state
          mountPath: /var/lib/tailscale
      volumes:
      - name: wireguard-config
        persistentVolumeClaim:
          claimName: wireguard-config-pvc
      - name: tailscale-state
        persistentVolumeClaim:
          claimName: tailscale-state-pvc
```

### API Deployment

```yaml
# api-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: proton-vpn-api
  namespace: proton-vpn
spec:
  replicas: 2
  selector:
    matchLabels:
      app: proton-vpn-api
  template:
    metadata:
      labels:
        app: proton-vpn-api
    spec:
      containers:
      - name: api
        image: your-registry/proton-vpn-api:latest
        ports:
        - containerPort: 8080
        env:
        - name: RUST_LOG
          value: "info"
        volumeMounts:
        - name: docker-sock
          mountPath: /var/run/docker.sock
      volumes:
      - name: docker-sock
        hostPath:
          path: /var/run/docker.sock
---
apiVersion: v1
kind: Service
metadata:
  name: proton-vpn-api
  namespace: proton-vpn
spec:
  selector:
    app: proton-vpn-api
  ports:
  - port: 8080
    targetPort: 8080
  type: ClusterIP
```

### Apply Configuration

```bash
kubectl apply -f namespace.yaml
kubectl apply -f secrets.yaml
kubectl apply -f vpn-deployment.yaml
kubectl apply -f api-deployment.yaml
```

**Note:** After deploying to Kubernetes, you still need to approve the exit node in Tailscale Admin Console.

## Cloud Deployment

### AWS EC2 Deployment

1. **Launch EC2 Instance:**
   - t3.medium or larger
   - Ubuntu 22.04 LTS
   - Security Group: Allow 22, 80, 443, 8080, 3000, UDP 41641, UDP 51820

2. **Install Docker:**
```bash
# Update system
sudo apt-get update && sudo apt-get upgrade -y

# Install Docker
sudo apt-get install -y docker.io docker-compose-plugin

# Start Docker
sudo systemctl start docker
sudo systemctl enable docker

# Add user to docker group
sudo usermod -aG docker ubuntu
```

3. **Deploy:**
```bash
git clone https://github.com/yourusername/proton-over-tailscale-proxy.git
cd proton-over-tailscale-proxy

# Set up environment
cp config/.env.example config/.env
# Edit config/.env with your credentials

# Deploy
docker compose up -d
```

4. **Enable Exit Node in Tailscale Console**

5. **Set up SSL with Let's Encrypt:**
```bash
sudo apt-get install -y certbot
sudo certbot certonly --standalone -d yourdomain.com
```

### DigitalOcean Droplet

1. **Create Droplet:**
   - 2 GB RAM / 1 CPU minimum
   - Ubuntu 22.04
   - Enable IPv6

2. **Install and Deploy:**
```bash
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker $USER
newgrp docker

# Clone and deploy
git clone https://github.com/yourusername/proton-over-tailscale-proxy.git
cd proton-over-tailscale-proxy

# Configure and deploy
cp config/.env.example config/.env
# Edit config/.env

docker compose up -d
```

3. **Enable Exit Node in Tailscale Console**

## Security Considerations

### 1. Environment Variables

- Never commit `.env` files
- Use Docker secrets or Kubernetes secrets
- Rotate credentials regularly
- Use strong passwords

### 2. Network Security

```bash
# Configure UFW (Ubuntu)
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow ssh
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw allow 8080/tcp  # If not using reverse proxy
sudo ufw allow 41641/udp  # Tailscale
sudo ufw allow 51820/udp  # WireGuard
sudo ufw enable
```

### 3. Container Security

```yaml
# Add to docker-compose.yml
services:
  vpn:
    read_only: true
    security_opt:
      - no-new-privileges:true
    tmpfs:
      - /tmp:noexec,nosuid,size=100m
```

### 4. Tailscale ACLs

Restrict who can use your exit node:

```json
{
  "acls": [
    {
      "action": "accept",
      "src": ["group:trusted-users"],
      "dst": ["tag:exit-node:*"]
    }
  ],
  "tagOwners": {
    "tag:exit-node": ["autogroup:admin"]
  }
}
```

### 5. Fail2Ban

```bash
sudo apt-get install -y fail2ban

# Create /etc/fail2ban/jail.local
cat > /etc/fail2ban/jail.local << 'EOF'
[DEFAULT]
bantime = 3600
findtime = 600
maxretry = 3

[sshd]
enabled = true

[proton-vpn-api]
enabled = true
port = 8080
filter = proton-vpn-api
logpath = /var/log/proton-vpn-api.log
maxretry = 5
EOF

sudo systemctl restart fail2ban
```

## Monitoring

### Prometheus + Grafana Setup

```yaml
# Add to docker-compose.yml
  prometheus:
    image: prom/prometheus:latest
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
    ports:
      - "9090:9090"

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3001:3000"
    volumes:
      - grafana-storage:/var/lib/grafana
```

### Exit Node Monitoring Script

```bash
#!/bin/bash
# exit-node-health-check.sh

API_URL="http://localhost:8080/exit-node"
WEBHOOK_URL="https://hooks.slack.com/services/YOUR/WEBHOOK/URL"

status=$(curl -s "$API_URL")
approved=$(echo "$status" | jq -r '.approved')
enabled=$(echo "$status" | jq -r '.enabled')

if [ "$approved" != "true" ]; then
    curl -X POST -H 'Content-type: application/json' \
        --data '{"text":"🚨 Exit node not approved in Tailscale!"}' \
        "$WEBHOOK_URL"
fi

if [ "$enabled" != "true" ]; then
    curl -X POST -H 'Content-type: application/json' \
        --data '{"text":"⚠️ Exit node is disabled!"}' \
        "$WEBHOOK_URL"
fi
```

Add to crontab:
```bash
*/5 * * * * /opt/proton-vpn-exit-node/exit-node-health-check.sh
```

## Backup and Recovery

### Backup Script

```bash
#!/bin/bash
# backup.sh

BACKUP_DIR="/opt/backups/proton-vpn"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

mkdir -p "$BACKUP_DIR"

# Backup configurations
tar czf "$BACKUP_DIR/config_$TIMESTAMP.tar.gz" ./config/

# Backup Docker volumes
docker run --rm -v proton-vpn-exit-node_vpn-config:/data -v "$BACKUP_DIR:/backup" alpine tar czf /backup/vpn-config_$TIMESTAMP.tar.gz -C /data .
docker run --rm -v proton-vpn-exit-node_tailscale-state:/data -v "$BACKUP_DIR:/backup" alpine tar czf /backup/tailscale-state_$TIMESTAMP.tar.gz -C /data .

# Backup environment file (keep it secure!)
cp .env "$BACKUP_DIR/env_$TIMESTAMP.backup"
chmod 600 "$BACKUP_DIR/env_$TIMESTAMP.backup"

# Cleanup old backups (keep 7 days)
find "$BACKUP_DIR" -name "*.tar.gz" -mtime +7 -delete
find "$BACKUP_DIR" -name "*.backup" -mtime +7 -delete

echo "Backup completed: $TIMESTAMP"
```

### Restore Script

```bash
#!/bin/bash
# restore.sh

BACKUP_DIR="/opt/backups/proton-vpn"
BACKUP_FILE=$1

if [ -z "$BACKUP_FILE" ]; then
    echo "Usage: $0 <backup_file>"
    exit 1
fi

# Stop services
docker compose down

# Restore configuration
tar xzf "$BACKUP_DIR/$BACKUP_FILE" -C /

# Restore volumes
docker run --rm -v proton-vpn-exit-node_vpn-config:/data -v "$BACKUP_DIR:/backup" alpine tar xzf "/backup/vpn-config_$BACKUP_FILE" -C /data
docker run --rm -v proton-vpn-exit-node_tailscale-state:/data -v "$BACKUP_DIR:/backup" alpine tar xzf "/backup/tailscale-state_$BACKUP_FILE" -C /data

# Start services
docker compose up -d

echo "Restore completed from: $BACKUP_FILE"
echo "Remember to re-approve exit node in Tailscale Admin Console!"
```

## Troubleshooting Production Issues

### Container Won't Start

```bash
# Check logs
docker compose logs --tail=100 vpn

# Check resource usage
docker stats

# Verify environment variables
docker compose config
```

### VPN Connection Drops

```bash
# Check network connectivity
docker exec proton-vpn ping -c 4 1.1.1.1

# Check WireGuard status
docker exec proton-vpn wg show

# Restart VPN container
docker compose restart vpn
```

### Exit Node Not Working

1. **Check Approval Status:**
   ```bash
   curl http://localhost:8080/exit-node
   ```

2. **Verify in Tailscale Console:**
   - Go to https://login.tailscale.com/admin/machines
   - Ensure "Use as exit node" is enabled

3. **Check Tailscale Logs:**
   ```bash
   docker exec proton-vpn tailscale status
   docker exec proton-vpn tailscale netcheck
   ```

4. **Verify NAT Configuration:**
   ```bash
   docker exec proton-vpn iptables -t nat -L
   ```

### Clients Cannot Connect

1. **Verify ACL Rules:**
   - Check Tailscale ACL policy allows exit node access

2. **Check Client Tailscale Version:**
   ```bash
   tailscale version
   ```

3. **Verify Network Connectivity:**
   ```bash
   tailscale ping proton-vpn-exit
   ```

### API Unresponsive

```bash
# Check if API is listening
netstat -tlnp | grep 8080

# Check container health
docker ps --filter name=api

# Restart API
docker compose restart api
```
