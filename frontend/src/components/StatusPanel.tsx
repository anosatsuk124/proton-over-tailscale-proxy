import type { StatusPanelProps, ConnectionStatus } from '../types'

// Component to display VPN and Tailscale connection status
export function StatusPanel({ status, loading, error }: StatusPanelProps) {
  // Helper function to get status CSS class
  const getStatusClass = (connectionStatus: ConnectionStatus | undefined): string => {
    switch (connectionStatus) {
      case 'connected':
        return 'status-connected'
      case 'disconnected':
        return 'status-disconnected'
      case 'connecting':
        return 'status-unknown'
      default:
        return 'status-unknown'
    }
  }

  // Helper function to format status text
  const formatStatus = (connectionStatus: ConnectionStatus | undefined): string => {
    if (!connectionStatus) return 'Unknown'
    return connectionStatus.charAt(0).toUpperCase() + connectionStatus.slice(1)
  }

  if (loading && !status) {
    return <div className="loading">Loading status...</div>
  }

  if (error) {
    return <div className="error">Error: {error}</div>
  }

  return (
    <div className="status-container">
      <div className="status-item">
        <span className="status-label">ProtonVPN</span>
        <span className={`status-value ${getStatusClass(status?.protonvpn)}`}>
          {formatStatus(status?.protonvpn)}
        </span>
      </div>

      <div className="status-item">
        <span className="status-label">Tailscale</span>
        <span className={`status-value ${getStatusClass(status?.tailscale)}`}>
          {formatStatus(status?.tailscale)}
        </span>
      </div>

      {status?.lastUpdated && (
        <div className="status-item">
          <span className="status-label">Last Updated</span>
          <span className="status-value status-unknown">
            {new Date(status.lastUpdated).toLocaleTimeString()}
          </span>
        </div>
      )}
    </div>
  )
}
