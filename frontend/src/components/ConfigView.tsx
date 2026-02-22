import { useState, useEffect } from 'react'
import { useApi } from '../hooks/useApi'

// Component to view and edit configuration
export function ConfigView() {
  const { config, loading, updateConfig, refreshConfig } = useApi()
  const [isEditing, setIsEditing] = useState(false)
  const [formData, setFormData] = useState({
    protonvpnServer: '',
    tailscaleHostname: '',
    autoConnect: false,
    advertiseExitNode: false,
  })
  const [saveStatus, setSaveStatus] = useState<string | null>(null)

  // Load config into form when available
  useEffect(() => {
    if (config) {
      setFormData({
        protonvpnServer: config.protonvpnServer || '',
        tailscaleHostname: config.tailscaleHostname || '',
        autoConnect: config.autoConnect || false,
        advertiseExitNode: config.advertiseExitNode || false,
      })
    }
  }, [config])

  // Handle input changes
  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value, type, checked } = e.target
    setFormData(prev => ({
      ...prev,
      [name]: type === 'checkbox' ? checked : value,
    }))
    setSaveStatus(null)
  }

  // Handle form submission
  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    
    const success = await updateConfig(formData)
    
    if (success) {
      setSaveStatus('Configuration saved successfully')
      setIsEditing(false)
      refreshConfig()
    } else {
      setSaveStatus('Failed to save configuration')
    }
    
    // Clear status after 3 seconds
    setTimeout(() => setSaveStatus(null), 3000)
  }

  // Handle cancel edit
  const handleCancel = () => {
    if (config) {
      setFormData({
        protonvpnServer: config.protonvpnServer || '',
        tailscaleHostname: config.tailscaleHostname || '',
        autoConnect: config.autoConnect || false,
        advertiseExitNode: config.advertiseExitNode || false,
      })
    }
    setIsEditing(false)
    setSaveStatus(null)
  }

  if (loading && !config) {
    return <div className="loading">Loading configuration...</div>
  }

  return (
    <div className="config-container">
      {saveStatus && (
        <div 
          style={{ 
            padding: '0.5rem', 
            borderRadius: '4px',
            backgroundColor: saveStatus.includes('success') ? 'rgba(76, 175, 80, 0.2)' : 'rgba(244, 67, 54, 0.2)',
            color: saveStatus.includes('success') ? '#4caf50' : '#f44336',
            marginBottom: '0.5rem'
          }}
        >
          {saveStatus}
        </div>
      )}

      {isEditing ? (
        <form onSubmit={handleSubmit}>
          <div className="config-item">
            <label className="config-label">ProtonVPN Server</label>
            <input
              type="text"
              name="protonvpnServer"
              value={formData.protonvpnServer}
              onChange={handleChange}
              className="config-input"
              placeholder="e.g., nl-free-01.protonvpn.com"
            />
          </div>

          <div className="config-item">
            <label className="config-label">Tailscale Hostname</label>
            <input
              type="text"
              name="tailscaleHostname"
              value={formData.tailscaleHostname}
              onChange={handleChange}
              className="config-input"
              placeholder="e.g., my-device"
            />
          </div>

          <div className="config-item">
            <label className="config-label" style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
              <input
                type="checkbox"
                name="advertiseExitNode"
                checked={formData.advertiseExitNode}
                onChange={handleChange}
                style={{ width: 'auto' }}
              />
              Advertise as exit node
            </label>
          </div>

          <div className="config-item">
            <label className="config-label" style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
              <input
                type="checkbox"
                name="autoConnect"
                checked={formData.autoConnect}
                onChange={handleChange}
                style={{ width: 'auto' }}
              />
              Auto-connect on startup
            </label>
          </div>

          <div className="controls" style={{ marginTop: '1rem' }}>
            <button 
              type="submit" 
              className="btn btn-connect"
              disabled={loading}
            >
              {loading ? 'Saving...' : 'Save'}
            </button>
            <button 
              type="button" 
              className="btn btn-refresh"
              onClick={handleCancel}
              disabled={loading}
            >
              Cancel
            </button>
          </div>
        </form>
      ) : (
        <>
          <div className="config-item">
            <span className="config-label">ProtonVPN Server</span>
            <span className="config-value">
              {config?.protonvpnServer || 'Not configured'}
            </span>
          </div>

          <div className="config-item">
            <span className="config-label">Tailscale Hostname</span>
            <span className="config-value">
              {config?.tailscaleHostname || 'Not configured'}
            </span>
          </div>

          <div className="config-item">
            <span className="config-label">Advertise Exit Node</span>
            <span className="config-value">
              {config?.advertiseExitNode ? 'Yes' : 'No'}
            </span>
          </div>

          <div className="config-item">
            <span className="config-label">Auto-connect</span>
            <span className="config-value">
              {config?.autoConnect ? 'Enabled' : 'Disabled'}
            </span>
          </div>

          <button 
            className="btn btn-refresh"
            onClick={() => setIsEditing(true)}
            style={{ marginTop: '1rem' }}
          >
            Edit Configuration
          </button>
        </>
      )}
    </div>
  )
}