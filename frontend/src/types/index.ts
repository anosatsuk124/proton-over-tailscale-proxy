// Status types for VPN and Tailscale connection states
export type ConnectionStatus = 'connected' | 'disconnected' | 'connecting' | 'unknown'

// System status response from backend
export interface SystemStatus {
  protonvpn: ConnectionStatus
  tailscale: ConnectionStatus
  lastUpdated: string
}

// Log entry structure
export interface LogEntry {
  timestamp: string
  level: 'info' | 'warn' | 'error' | 'debug'
  message: string
  source: string
}

// API response wrapper
export interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: string
}

// Configuration structure
export interface Config {
  protonvpnServer: string
  tailscaleHostname: string
  proxyPort: number
  autoConnect: boolean
}

// Action types for control buttons
export type ActionType = 'connect' | 'disconnect' | 'restart'

// Props for reusable components
export interface StatusPanelProps {
  status: SystemStatus | null
  loading: boolean
  error: string | null
}

export interface ControlButtonsProps {
  onAction: () => void
}

export interface LogViewerProps {
  maxLines?: number
  autoScroll?: boolean
}
