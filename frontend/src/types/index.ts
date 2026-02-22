// Status types for VPN and Tailscale connection states
export type ConnectionStatus = 'connected' | 'disconnected' | 'connecting' | 'unknown'

// Exit node status types
export type ExitNodeStatus = 'advertised' | 'not_advertised' | 'pending' | 'approved' | 'unknown'

// Client information for connected devices
export interface ConnectedClient {
  id: string
  name: string
  ip: string
  connectedAt: string
  lastSeen: string
}

// System status response from backend
export interface SystemStatus {
  protonvpn: ConnectionStatus
  tailscale: ConnectionStatus
  exitNode: ExitNodeStatus
  exitNodeEnabled: boolean
  exitNodeApproved: boolean
  connectedClients: ConnectedClient[]
  tailscaleIP: string | null
  protonvpnIP: string | null
  connectionQuality: 'excellent' | 'good' | 'fair' | 'poor' | 'unknown'
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
  autoConnect: boolean
  advertiseExitNode: boolean
}

// Action types for control buttons
export type ActionType = 'enable_exit_node' | 'disable_exit_node' | 'approve_exit_node' | 'restart'

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
