import { useState } from 'react'
import { useApi } from '../hooks/useApi'
import type { ControlButtonsProps } from '../types'

// Component for connect/disconnect/restart controls
export function ControlButtons({ onAction }: ControlButtonsProps) {
  const { executeAction, status, loading } = useApi()
  const [actionInProgress, setActionInProgress] = useState<string | null>(null)

  // Handle button click
  const handleAction = async (action: 'connect' | 'disconnect' | 'restart') => {
    setActionInProgress(action)
    
    const success = await executeAction(action)
    
    if (success) {
      // Notify parent component to refresh status
      onAction()
    }
    
    setActionInProgress(null)
  }

  // Determine if VPN is currently connected
  const isConnected = status?.protonvpn === 'connected'
  const isConnecting = status?.protonvpn === 'connecting'

  return (
    <div className="controls">
      <button
        className="btn btn-connect"
        onClick={() => handleAction('connect')}
        disabled={loading || isConnected || isConnecting || actionInProgress !== null}
      >
        {actionInProgress === 'connect' 
          ? 'Connecting...' 
          : isConnecting 
            ? 'Connecting...' 
            : 'Connect'}
      </button>

      <button
        className="btn btn-disconnect"
        onClick={() => handleAction('disconnect')}
        disabled={loading || !isConnected || actionInProgress !== null}
      >
        {actionInProgress === 'disconnect' ? 'Disconnecting...' : 'Disconnect'}
      </button>

      <button
        className="btn btn-refresh"
        onClick={() => handleAction('restart')}
        disabled={loading || actionInProgress !== null}
      >
        {actionInProgress === 'restart' ? 'Restarting...' : 'Restart Service'}
      </button>
    </div>
  )
}
