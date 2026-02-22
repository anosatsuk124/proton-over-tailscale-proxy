import { useState } from 'react'
import { useApi } from '../hooks/useApi'
import type { ControlButtonsProps } from '../types'

// Component for exit node control buttons
export function ControlButtons({ onAction }: ControlButtonsProps) {
  const { executeAction, status, loading } = useApi()
  const [actionInProgress, setActionInProgress] = useState<string | null>(null)

  // Handle button click for exit node actions
  const handleAction = async (action: 'enable_exit_node' | 'disable_exit_node' | 'approve_exit_node' | 'restart') => {
    setActionInProgress(action)
    
    const success = await executeAction(action)
    
    if (success) {
      // Notify parent component to refresh status
      onAction()
    }
    
    setActionInProgress(null)
  }

  // Determine exit node states
  const isExitNodeEnabled = status?.exitNodeEnabled || false
  const isExitNodeApproved = status?.exitNodeApproved || false
  const isExitNodeAdvertised = status?.exitNode === 'advertised' || status?.exitNode === 'approved'

  return (
    <div className="controls">
      <div className="control-group">
        <h3>Exit Node Controls</h3>
        
        <button
          className="btn btn-connect"
          onClick={() => handleAction('enable_exit_node')}
          disabled={loading || isExitNodeEnabled || actionInProgress !== null}
        >
          {actionInProgress === 'enable_exit_node' 
            ? 'Enabling...' 
            : 'Enable Exit Node'}
        </button>

        <button
          className="btn btn-disconnect"
          onClick={() => handleAction('disable_exit_node')}
          disabled={loading || !isExitNodeEnabled || actionInProgress !== null}
        >
          {actionInProgress === 'disable_exit_node' 
            ? 'Disabling...' 
            : 'Disable Exit Node'}
        </button>

        {!isExitNodeApproved && isExitNodeAdvertised && (
          <button
            className="btn btn-approve"
            onClick={() => handleAction('approve_exit_node')}
            disabled={loading || actionInProgress !== null}
          >
            {actionInProgress === 'approve_exit_node' 
              ? 'Approving...' 
              : 'Approve Exit Node'}
          </button>
        )}
      </div>

      <div className="control-group">
        <h3>Service Controls</h3>
        
        <button
          className="btn btn-refresh"
          onClick={() => handleAction('restart')}
          disabled={loading || actionInProgress !== null}
        >
          {actionInProgress === 'restart' ? 'Restarting...' : 'Restart Service'}
        </button>
      </div>
    </div>
  )
}
