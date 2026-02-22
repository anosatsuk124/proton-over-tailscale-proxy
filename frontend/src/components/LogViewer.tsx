import { useRef, useEffect } from 'react'
import { useLogs } from '../hooks/useLogs'

// Component to display and manage system logs including exit node logs
export function LogViewer({ maxLines = 100 }: { maxLines?: number }) {
  const { 
    logs, 
    loading, 
    error, 
    autoRefresh, 
    refreshLogs, 
    clearLogs, 
    exportLogs,
    toggleAutoRefresh 
  } = useLogs(maxLines)
  
  const containerRef = useRef<HTMLDivElement>(null)

  // Auto-scroll to bottom when new logs arrive
  useEffect(() => {
    if (containerRef.current && autoRefresh) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight
    }
  }, [logs, autoRefresh])

  // Helper function to get log level CSS class
  const getLevelClass = (level: string): string => {
    switch (level) {
      case 'info':
        return 'log-level-info'
      case 'error':
        return 'log-level-error'
      case 'warn':
        return 'log-level-warn'
      default:
        return ''
    }
  }

  // Helper function to check if log is exit node related
  const isExitNodeLog = (message: string, source: string): boolean => {
    const exitNodeKeywords = [
      'exit node',
      'exit-node',
      'advertise',
      'approve',
      'client connected',
      'client disconnected',
      'tailscale up',
      'tailscale down'
    ]
    const lowerMessage = message.toLowerCase()
    const lowerSource = source.toLowerCase()
    return exitNodeKeywords.some(keyword => 
      lowerMessage.includes(keyword) || lowerSource.includes(keyword)
    )
  }

  return (
    <div className="log-viewer">
      <div className="log-controls">
        <button 
          className="btn btn-small btn-refresh" 
          onClick={refreshLogs}
          disabled={loading}
        >
          {loading ? 'Loading...' : 'Refresh'}
        </button>
        
        <button 
          className={`btn btn-small ${autoRefresh ? 'btn-connect' : 'btn-refresh'}`}
          onClick={toggleAutoRefresh}
        >
          {autoRefresh ? 'Auto: ON' : 'Auto: OFF'}
        </button>
        
        <button 
          className="btn btn-small btn-refresh"
          onClick={clearLogs}
        >
          Clear
        </button>
        
        <button 
          className="btn btn-small btn-refresh"
          onClick={exportLogs}
          disabled={logs.length === 0}
        >
          Export
        </button>
      </div>

      {error && <div className="error" style={{ marginBottom: '0.5rem' }}>Error: {error}</div>}

      <div className="log-container" ref={containerRef}>
        {logs.length === 0 ? (
          <div className="log-entry">
            <span className="log-timestamp">--:--:--</span>
            <span>No logs available</span>
          </div>
        ) : (
          logs.map((log, index) => {
            const isExitNode = isExitNodeLog(log.message, log.source)
            return (
              <div 
                key={`${log.timestamp}-${index}`} 
                className={`log-entry ${isExitNode ? 'log-entry-exit-node' : ''}`}
              >
                <span className="log-timestamp">
                  {new Date(log.timestamp).toLocaleTimeString()}
                </span>
                <span className={getLevelClass(log.level)}>
                  [{log.level.toUpperCase()}]
                </span>
                {isExitNode && (
                  <span className="log-badge">EXIT-NODE</span>
                )}
                <span> {log.source}: {log.message}</span>
              </div>
            )
          })
        )}
      </div>

      <div style={{ marginTop: '0.5rem', fontSize: '0.75rem', opacity: 0.7 }}>
        Showing {logs.length} lines {autoRefresh && '(auto-refreshing)'}
      </div>
    </div>
  )
}
