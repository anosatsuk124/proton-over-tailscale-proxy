import type { StatusPanelProps, ConnectionStatus, ExitNodeStatus } from '../types'

// Component to display Exit Node and VPN connection status
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

  // Helper function to get exit node status CSS class
  const getExitNodeStatusClass = (exitNodeStatus: ExitNodeStatus | undefined): string => {
    switch (exitNodeStatus) {
      case 'approved':
        return 'status-connected'
      case 'advertised':
        return 'status-unknown'
      case 'not_advertised':
        return 'status-disconnected'
      case 'pending':
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

  // Helper function to format exit node status text
  const formatExitNodeStatus = (exitNodeStatus: ExitNodeStatus | undefined): string => {
    if (!exitNodeStatus) return 'Unknown'
    return exitNodeStatus
      .split('_')
      .map(word => word.charAt(0).toUpperCase() + word.slice(1))
      .join(' ')
  }

  // Helper function to format connection quality
  const formatConnectionQuality = (quality: string | undefined): string => {
    if (!quality || quality === 'unknown') return 'Unknown'
    return quality.charAt(0).toUpperCase() + quality.slice(1)
  }

  if (loading && !status) {
    return <div className="loading">Loading status...</div>
  }

  if (error) {
    return <div className="error">Error: {error}</div>
  }

  return (
    <div className="status-container">
      <div className="status-section">
        <h3>Exit Node Status</h3>
        
        <div className="status-item">
          <span className="status-label">Status</span>
          <span className={`status-value ${getExitNodeStatusClass(status?.exitNode)}`}>
            {formatExitNodeStatus(status?.exitNode)}
          </span>
        </div>

        <div className="status-item">
          <span className="status-label">Enabled</span>
          <span className={`status-value ${status?.exitNodeEnabled ? 'status-connected' : 'status-disconnected'}`}>
            {status?.exitNodeEnabled ? 'Yes' : 'No'}
          </span>
        </div>

        <div className="status-item">
          <span className="status-label">Approved</span>
          <span className={`status-value ${status?.exitNodeApproved ? 'status-connected' : 'status-disconnected'}`}>
            {status?.exitNodeApproved ? 'Yes' : 'No'}
          </span>
        </div>

        <div className="status-item">
          <span className="status-label">Connected Clients</span>
          <span className="status-value">
            {status?.connectedClients?.length || 0}
          </span>
        </div>
      </div>

      <div className="status-section">
        <h3>Network Status</h3>
        
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
      </div>

      <div className="status-section">
        <h3>IP Addresses</h3>
        
        <div className="status-item">
          <span className="status-label">Tailscale IP</span>
          <span className="status-value">
            {status?.tailscaleIP || 'Not available'}
          </span>
        </div>

        <div className="status-item">
          <span className="status-label">ProtonVPN IP</span>
          <span className="status-value">
            {status?.protonvpnIP || 'Not available'}
          </span>
        </div>
      </div>

      <div className="status-section">
        <h3>Connection Quality</h3>
        
        <div className="status-item">
          <span className="status-label">Quality</span>
          <span className={`status-value status-${status?.connectionQuality || 'unknown'}`}>
            {formatConnectionQuality(status?.connectionQuality)}
          </span>
        </div>

        {status?.lastUpdated && (
          <div className="status-item">
            <span className="status-label">Last Updated</span>
            <span className="status-value">
              {new Date(status.lastUpdated).toLocaleTimeString()}
            </span>
          </div>
        )}
      </div>

      {status?.connectedClients && status.connectedClients.length > 0 && (
        <div className="status-section">
          <h3>Connected Clients ({status.connectedClients.length})</h3>
          <div className="clients-list">
            {status.connectedClients.map((client) => (
              <div key={client.id} className="client-item">
                <span className="client-name">{client.name}</span>
                <span className="client-ip">{client.ip}</span>
                <span className="client-time">
                  Connected: {new Date(client.connectedAt).toLocaleTimeString()}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  )
}
