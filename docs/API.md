# API Documentation

This document describes the REST API endpoints available in the ProtonVPN Tailscale Exit Node system.

## Base URL

- Development: `http://localhost:8080`
- Production: `https://your-domain.com`

## Authentication

Currently, the API uses simple token-based authentication (optional). Include the token in the Authorization header:

```
Authorization: Bearer YOUR_API_TOKEN
```

## Response Format

All responses are JSON with the following structure:

### Success Response
```json
{
  "success": true,
  "data": { ... },
  "message": "Operation completed successfully"
}
```

### Error Response
```json
{
  "success": false,
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error description"
  }
}
```

## Endpoints

### Health Check

Check if the API server is running.

**Endpoint:** `GET /health`

**Response:**
- `200 OK` - Server is healthy

**Example:**
```bash
curl http://localhost:8080/health
```

---

### Get Status

Retrieve the current connection status of VPN and Tailscale containers.

**Endpoint:** `GET /status`

**Response:**
```json
{
  "connected": true,
  "vpn_container": {
    "name": "proton-vpn",
    "running": true,
    "status": "running",
    "image": "proton-vpn:latest"
  },
  "tailscale_container": {
    "name": "tailscale",
    "running": true,
    "status": "running",
    "image": "tailscale:latest"
  },
  "last_error": null
}
```

**Example:**
```bash
curl http://localhost:8080/status
```

---

### Get Exit Node Status

Retrieve the current exit node status and configuration.

**Endpoint:** `GET /exit-node`

**Response:**
```json
{
  "enabled": true,
  "advertised": true,
  "approved": true,
  "hostname": "proton-vpn-exit",
  "tailscale_ip": "100.x.y.z",
  "connected_clients": 3,
  "last_updated": "2024-01-15T10:30:00Z"
}
```

**Fields:**
- `enabled`: Whether exit node functionality is enabled
- `advertised`: Whether the node is advertising as an exit node
- `approved`: Whether approved in Tailscale admin console
- `hostname`: The hostname in Tailscale network
- `tailscale_ip`: The Tailscale IP address
- `connected_clients`: Number of clients using this exit node
- `last_updated`: Last status update timestamp

**Example:**
```bash
curl http://localhost:8080/exit-node
```

---

### Update Exit Node

Enable or disable the exit node functionality.

**Endpoint:** `POST /exit-node`

**Request Body:**
```json
{
  "enabled": true
}
```

**Parameters:**
- `enabled` (required): Boolean to enable or disable exit node

**Response:**
```json
{
  "success": true,
  "message": "Exit node enabled successfully",
  "data": {
    "enabled": true,
    "advertised": true
  }
}
```

**Error Codes:**
- `400 Bad Request`: Invalid request body
- `500 Internal Server Error`: Failed to update exit node configuration

**Example:**
```bash
# Enable exit node
curl -X POST http://localhost:8080/exit-node \
  -H "Content-Type: application/json" \
  -d '{"enabled": true}'

# Disable exit node
curl -X POST http://localhost:8080/exit-node \
  -H "Content-Type: application/json" \
  -d '{"enabled": false}'
```

---

### Connect to VPN

Start the VPN connection with optional server selection.

**Endpoint:** `POST /connect`

**Request Body:**
```json
{
  "server": "US-FREE#1",
  "protocol": "wireguard"
}
```

**Parameters:**
- `server` (optional): ProtonVPN server to connect to (e.g., "US-FREE#1", "JP-FREE#1")
- `protocol` (optional): VPN protocol to use (default: "wireguard")

**Response:**
```json
{
  "success": true,
  "message": "VPN connection initiated"
}
```

**Error Codes:**
- `400 Bad Request`: Invalid server or protocol
- `409 Conflict`: VPN already connected
- `500 Internal Server Error`: Docker or configuration error

**Example:**
```bash
curl -X POST http://localhost:8080/connect \
  -H "Content-Type: application/json" \
  -d '{
    "server": "NL-FREE#1",
    "protocol": "wireguard"
  }'
```

---

### Disconnect from VPN

Stop the VPN connection.

**Endpoint:** `POST /disconnect`

**Response:**
```json
{
  "success": true,
  "message": "VPN disconnected successfully"
}
```

**Error Codes:**
- `409 Conflict`: VPN not connected
- `500 Internal Server Error`: Docker error

**Example:**
```bash
curl -X POST http://localhost:8080/disconnect
```

---

### Get Logs

Retrieve recent logs from the VPN container.

**Endpoint:** `GET /logs`

**Query Parameters:**
- `lines` (optional): Number of log lines to return (default: 100, max: 1000)
- `since` (optional): Only return logs since this timestamp (ISO 8601 format)

**Response:**
```json
{
  "logs": [
    {
      "timestamp": "2024-01-15T10:30:00Z",
      "level": "info",
      "message": "WireGuard interface wg0 created"
    },
    {
      "timestamp": "2024-01-15T10:30:01Z",
      "level": "info",
      "message": "Connected to ProtonVPN server US-FREE#1"
    }
  ]
}
```

**Example:**
```bash
# Get last 50 log lines
curl "http://localhost:8080/logs?lines=50"

# Get logs from specific time
curl "http://localhost:8080/logs?since=2024-01-15T00:00:00Z"
```

---

### Stream Logs

Stream logs in real-time via WebSocket.

**Endpoint:** `GET /logs/stream` (WebSocket)

**Protocol:** WebSocket (`ws://` or `wss://`)

**Message Format:**
```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "level": "info",
  "source": "wireguard",
  "message": "Received handshake response"
}
```

**Example (JavaScript):**
```javascript
const ws = new WebSocket('ws://localhost:8080/logs/stream');

ws.onmessage = (event) => {
  const log = JSON.parse(event.data);
  console.log(`[${log.level}] ${log.message}`);
};

ws.onclose = () => {
  console.log('Log stream closed');
};
```

---

### Get Configuration

Retrieve the current system configuration.

**Endpoint:** `GET /config`

**Response:**
```json
{
  "api_port": 8080,
  "frontend_port": 3000,
  "default_server": "US-FREE#1",
  "log_level": "info",
  "tailscale": {
    "hostname": "proton-vpn-exit",
    "advertise_exit_node": true
  },
  "features": {
    "auto_reconnect": true,
    "kill_switch": true
  }
}
```

**Note:** Sensitive information (passwords, auth keys) is redacted from the response.

**Example:**
```bash
curl http://localhost:8080/config
```

---

### Update Configuration

Update system configuration (requires authentication).

**Endpoint:** `POST /config`

**Request Body:**
```json
{
  "default_server": "JP-FREE#1",
  "log_level": "debug",
  "tailscale": {
    "hostname": "new-hostname",
    "advertise_exit_node": true
  },
  "features": {
    "auto_reconnect": true,
    "kill_switch": false
  }
}
```

**Response:**
```json
{
  "success": true,
  "message": "Configuration updated successfully"
}
```

**Error Codes:**
- `400 Bad Request`: Invalid configuration values
- `401 Unauthorized`: Authentication required
- `403 Forbidden`: Insufficient permissions

**Example:**
```bash
curl -X POST http://localhost:8080/config \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{
    "default_server": "NL-FREE#1",
    "log_level": "info",
    "tailscale": {
      "advertise_exit_node": true
    }
  }'
```

---

## Error Codes Reference

### HTTP Status Codes

| Code | Description |
|------|-------------|
| 200 | OK - Request successful |
| 201 | Created - Resource created successfully |
| 400 | Bad Request - Invalid request parameters |
| 401 | Unauthorized - Authentication required |
| 403 | Forbidden - Insufficient permissions |
| 404 | Not Found - Resource not found |
| 409 | Conflict - Resource state conflict |
| 422 | Unprocessable Entity - Validation error |
| 429 | Too Many Requests - Rate limit exceeded |
| 500 | Internal Server Error - Server error |
| 503 | Service Unavailable - Service temporarily unavailable |

### Application Error Codes

| Code | Description |
|------|-------------|
| `DOCKER_ERROR` | Docker daemon error |
| `CONFIG_ERROR` | Configuration error |
| `VPN_ERROR` | VPN connection error |
| `EXIT_NODE_ERROR` | Exit node configuration error |
| `AUTH_ERROR` | Authentication error |
| `VALIDATION_ERROR` | Request validation error |
| `NOT_FOUND` | Resource not found |
| `RATE_LIMIT` | Rate limit exceeded |

## Rate Limiting

API endpoints are rate-limited to prevent abuse:

- **Health Check**: No limit
- **Status/Logs**: 100 requests per minute
- **Connect/Disconnect**: 10 requests per minute
- **Exit Node Operations**: 10 requests per minute
- **Configuration**: 30 requests per minute

Rate limit headers are included in responses:
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1640995200
```

## CORS

The API supports Cross-Origin Resource Sharing (CORS) for browser-based clients. The following headers are configured:

- `Access-Control-Allow-Origin: *`
- `Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS`
- `Access-Control-Allow-Headers: Content-Type, Authorization`

Preflight requests (`OPTIONS`) are automatically handled.

## Versioning

The current API version is **v1**. Version is specified in the URL path:

```
http://localhost:8080/v1/status
```

For backward compatibility, unversioned endpoints default to the latest stable version.

## WebSocket Considerations

- WebSocket connections automatically close after 5 minutes of inactivity
- Maximum concurrent connections per client: 5
- Reconnection is handled automatically by the frontend

## SDK Examples

### Python

```python
import requests

BASE_URL = "http://localhost:8080"

class ProtonVPNExitNodeClient:
    def __init__(self, base_url: str = BASE_URL):
        self.base_url = base_url
    
    def get_status(self):
        response = requests.get(f"{self.base_url}/status")
        return response.json()
    
    def get_exit_node_status(self):
        response = requests.get(f"{self.base_url}/exit-node")
        return response.json()
    
    def enable_exit_node(self):
        response = requests.post(
            f"{self.base_url}/exit-node",
            json={"enabled": True}
        )
        return response.json()
    
    def disable_exit_node(self):
        response = requests.post(
            f"{self.base_url}/exit-node",
            json={"enabled": False}
        )
        return response.json()
    
    def connect(self, server="US-FREE#1"):
        response = requests.post(
            f"{self.base_url}/connect",
            json={"server": server, "protocol": "wireguard"}
        )
        return response.json()
    
    def disconnect(self):
        response = requests.post(f"{self.base_url}/disconnect")
        return response.json()

# Usage
client = ProtonVPNExitNodeClient()

# Check current status
status = client.get_status()
print(f"VPN Connected: {status['connected']}")

# Check exit node status
exit_node = client.get_exit_node_status()
print(f"Exit Node Enabled: {exit_node['enabled']}")
print(f"Connected Clients: {exit_node['connected_clients']}")

# Enable exit node
result = client.enable_exit_node()
print(result['message'])
```

### JavaScript/TypeScript

```typescript
class ProtonVPNExitNodeClient {
  private baseUrl: string;
  
  constructor(baseUrl: string = 'http://localhost:8080') {
    this.baseUrl = baseUrl;
  }
  
  async getStatus() {
    const response = await fetch(`${this.baseUrl}/status`);
    return response.json();
  }
  
  async getExitNodeStatus() {
    const response = await fetch(`${this.baseUrl}/exit-node`);
    return response.json();
  }
  
  async setExitNode(enabled: boolean) {
    const response = await fetch(`${this.baseUrl}/exit-node`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ enabled })
    });
    return response.json();
  }
  
  async connect(server: string = 'US-FREE#1') {
    const response = await fetch(`${this.baseUrl}/connect`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ server, protocol: 'wireguard' })
    });
    return response.json();
  }
  
  async disconnect() {
    const response = await fetch(`${this.baseUrl}/disconnect`, {
      method: 'POST'
    });
    return response.json();
  }
  
  streamLogs(callback: (log: any) => void) {
    const ws = new WebSocket(`${this.baseUrl.replace('http', 'ws')}/logs/stream`);
    ws.onmessage = (event) => callback(JSON.parse(event.data));
    return ws;
  }
}

// Usage
const client = new ProtonVPNExitNodeClient();

// Enable exit node and connect
async function setupExitNode() {
  await client.connect('NL-FREE#1');
  await client.setExitNode(true);
  
  const status = await client.getExitNodeStatus();
  console.log(`Exit node enabled: ${status.enabled}`);
  console.log(`Connected clients: ${status.connected_clients}`);
}

setupExitNode();
```

### Go

```go
package main

import (
    "bytes"
    "encoding/json"
    "fmt"
    "net/http"
)

type Client struct {
    baseURL string
    client  *http.Client
}

func NewClient(baseURL string) *Client {
    return &Client{
        baseURL: baseURL,
        client:  &http.Client{},
    }
}

func (c *Client) GetExitNodeStatus() (map[string]interface{}, error) {
    resp, err := c.client.Get(c.baseURL + "/exit-node")
    if err != nil {
        return nil, err
    }
    defer resp.Body.Close()
    
    var result map[string]interface{}
    if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
        return nil, err
    }
    
    return result, nil
}

func (c *Client) SetExitNode(enabled bool) error {
    payload := map[string]bool{"enabled": enabled}
    
    body, _ := json.Marshal(payload)
    resp, err := c.client.Post(
        c.baseURL+"/exit-node",
        "application/json",
        bytes.NewBuffer(body),
    )
    
    if err != nil {
        return err
    }
    defer resp.Body.Close()
    
    if resp.StatusCode != http.StatusOK {
        return fmt.Errorf("failed to update exit node: %d", resp.StatusCode)
    }
    
    return nil
}

func (c *Client) Connect(server string) error {
    payload := map[string]string{
        "server":   server,
        "protocol": "wireguard",
    }
    
    body, _ := json.Marshal(payload)
    resp, err := c.client.Post(
        c.baseURL+"/connect",
        "application/json",
        bytes.NewBuffer(body),
    )
    
    if err != nil {
        return err
    }
    defer resp.Body.Close()
    
    if resp.StatusCode != http.StatusOK {
        return fmt.Errorf("connection failed: %d", resp.StatusCode)
    }
    
    return nil
}

// Usage
func main() {
    client := NewClient("http://localhost:8080")
    
    // Connect and enable exit node
    if err := client.Connect("US-FREE#1"); err != nil {
        fmt.Printf("Error connecting: %v\n", err)
        return
    }
    
    if err := client.SetExitNode(true); err != nil {
        fmt.Printf("Error enabling exit node: %v\n", err)
        return
    }
    
    status, err := client.GetExitNodeStatus()
    if err != nil {
        fmt.Printf("Error getting status: %v\n", err)
        return
    }
    
    fmt.Printf("Exit node enabled: %v\n", status["enabled"])
    fmt.Printf("Connected clients: %v\n", status["connected_clients"])
}
```

## Exit Node Workflow

### Complete Setup Flow

```bash
# 1. Check initial status
curl http://localhost:8080/status

# 2. Connect to VPN
curl -X POST http://localhost:8080/connect \
  -H "Content-Type: application/json" \
  -d '{"server": "US-FREE#1"}'

# 3. Verify VPN is connected
curl http://localhost:8080/status

# 4. Check exit node status
curl http://localhost:8080/exit-node

# 5. Enable exit node (if not already enabled)
curl -X POST http://localhost:8080/exit-node \
  -H "Content-Type: application/json" \
  -d '{"enabled": true}'

# 6. Approve in Tailscale Admin Console
# Visit: https://login.tailscale.com/admin/machines
# Find your node and enable "Use as exit node"

# 7. Verify clients can connect
curl http://localhost:8080/exit-node
# Should show "approved": true and connected_clients > 0
```
