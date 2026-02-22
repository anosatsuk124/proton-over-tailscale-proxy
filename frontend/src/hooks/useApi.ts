import { useState, useEffect, useCallback } from 'react'
import type { SystemStatus, ApiResponse, Config } from '../types'

const API_BASE = '/api'

// Custom hook for API communication with the Rust backend
export function useApi() {
  const [status, setStatus] = useState<SystemStatus | null>(null)
  const [config, setConfig] = useState<Config | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // Fetch current system status
  const fetchStatus = useCallback(async () => {
    setLoading(true)
    setError(null)
    
    try {
      const response = await fetch(`${API_BASE}/status`)
      const result: ApiResponse<SystemStatus> = await response.json()
      
      if (result.success && result.data) {
        setStatus(result.data)
      } else {
        setError(result.error || 'Failed to fetch status')
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Network error')
    } finally {
      setLoading(false)
    }
  }, [])

  // Fetch configuration
  const fetchConfig = useCallback(async () => {
    try {
      const response = await fetch(`${API_BASE}/config`)
      const result: ApiResponse<Config> = await response.json()
      
      if (result.success && result.data) {
        setConfig(result.data)
      }
    } catch (err) {
      console.error('Failed to fetch config:', err)
    }
  }, [])

  // Execute an action (connect/disconnect/restart)
  const executeAction = useCallback(async (action: 'connect' | 'disconnect' | 'restart') => {
    setLoading(true)
    setError(null)
    
    try {
      const response = await fetch(`${API_BASE}/action`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ action }),
      })
      
      const result: ApiResponse<{ message: string }> = await response.json()
      
      if (!result.success) {
        setError(result.error || `Failed to ${action}`)
      }
      
      // Refresh status after action
      await fetchStatus()
      
      return result.success
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Network error')
      return false
    } finally {
      setLoading(false)
    }
  }, [fetchStatus])

  // Update configuration
  const updateConfig = useCallback(async (newConfig: Partial<Config>) => {
    setLoading(true)
    setError(null)
    
    try {
      const response = await fetch(`${API_BASE}/config`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(newConfig),
      })
      
      const result: ApiResponse<Config> = await response.json()
      
      if (result.success && result.data) {
        setConfig(result.data)
        return true
      } else {
        setError(result.error || 'Failed to update config')
        return false
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Network error')
      return false
    } finally {
      setLoading(false)
    }
  }, [])

  // Initial fetch on mount
  useEffect(() => {
    fetchStatus()
    fetchConfig()
  }, [fetchStatus, fetchConfig])

  // Auto-refresh status every 5 seconds
  useEffect(() => {
    const interval = setInterval(fetchStatus, 5000)
    return () => clearInterval(interval)
  }, [fetchStatus])

  return {
    status,
    config,
    loading,
    error,
    refreshStatus: fetchStatus,
    refreshConfig: fetchConfig,
    executeAction,
    updateConfig,
  }
}
