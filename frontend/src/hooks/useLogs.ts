import { useState, useEffect, useCallback, useRef } from 'react'
import type { LogEntry } from '../types'

const API_BASE = '/api'

// Custom hook for fetching and managing system logs
export function useLogs(maxLines: number = 100) {
  const [logs, setLogs] = useState<LogEntry[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [autoRefresh, setAutoRefresh] = useState(true)
  const lastTimestampRef = useRef<string | null>(null)

  // Fetch logs from backend
  const fetchLogs = useCallback(async () => {
    setLoading(true)
    setError(null)
    
    try {
      // Build query params for incremental updates
      const params = new URLSearchParams()
      params.append('limit', maxLines.toString())
      if (lastTimestampRef.current) {
        params.append('since', lastTimestampRef.current)
      }
      
      const response = await fetch(`${API_BASE}/logs?${params.toString()}`)
      
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`)
      }
      
      const newLogs: LogEntry[] = await response.json()
      
      if (newLogs.length > 0) {
        // Update last timestamp for next incremental fetch
        lastTimestampRef.current = newLogs[newLogs.length - 1].timestamp
        
        setLogs(prevLogs => {
          // Combine with existing logs and limit to maxLines
          const combined = [...prevLogs, ...newLogs]
          // Remove duplicates based on timestamp and message
          const unique = combined.filter((log, index, self) =>
            index === self.findIndex(l => 
              l.timestamp === log.timestamp && l.message === log.message
            )
          )
          // Keep only the last maxLines
          return unique.slice(-maxLines)
        })
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch logs')
    } finally {
      setLoading(false)
    }
  }, [maxLines])

  // Clear all logs
  const clearLogs = useCallback(() => {
    setLogs([])
    lastTimestampRef.current = null
  }, [])

  // Export logs as text file
  const exportLogs = useCallback(() => {
    const content = logs
      .map(log => `[${log.timestamp}] [${log.level.toUpperCase()}] ${log.source}: ${log.message}`)
      .join('\n')
    
    const blob = new Blob([content], { type: 'text/plain' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `proton-tailscale-logs-${new Date().toISOString().split('T')[0]}.txt`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
  }, [logs])

  // Toggle auto-refresh
  const toggleAutoRefresh = useCallback(() => {
    setAutoRefresh(prev => !prev)
  }, [])

  // Auto-refresh logs every 2 seconds
  useEffect(() => {
    if (!autoRefresh) return
    
    // Initial fetch
    fetchLogs()
    
    const interval = setInterval(fetchLogs, 2000)
    return () => clearInterval(interval)
  }, [autoRefresh, fetchLogs])

  return {
    logs,
    loading,
    error,
    autoRefresh,
    refreshLogs: fetchLogs,
    clearLogs,
    exportLogs,
    toggleAutoRefresh,
  }
}
