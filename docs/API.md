# API Documentation

This document describes the REST API endpoints available in the ProtonVPN over Tailscale Proxy system.

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
    "log_level": "info"
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
| `AUTH_ERROR` | Authentication error |
| `VALIDATION_ERROR` | Request validation error |
| `NOT_FOUND` | Resource not found |
| `RATE_LIMIT` | Rate limit exceeded |

## Rate Limiting

API endpoints are rate-limited to prevent abuse:

- **Health Check**: No limit
- **Status/Logs**: 100 requests per minute
- **Connect/Disconnect**: 10 requests per minute
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

def get_status():
    response = requests.get(f"{BASE_URL}/status")
    return response.json()

def connect(server="US-FREE#1"):
    response = requests.post(
        f"{BASE_URL}/connect",
        json={"server": server, "protocol": "wireguard"}
    )
    return response.json()

def disconnect():
    response = requests.post(f"{BASE_URL}/disconnect")
    return response.json()
```

### JavaScript/TypeScript

```typescript
class ProtonVPNClient {
  private baseUrl: string;
  
  constructor(baseUrl: string = 'http://localhost:8080') {
    this.baseUrl = baseUrl;
  }
  
  async getStatus() {
    const response = await fetch(`${this.baseUrl}/status`);
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
```
