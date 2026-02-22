import { StatusPanel } from './components/StatusPanel'
import { LogViewer } from './components/LogViewer'
import { ControlButtons } from './components/ControlButtons'
import { ConfigView } from './components/ConfigView'
import { useApi } from './hooks/useApi'

// Main application component for Tailscale Exit Node Dashboard
function App() {
  const { status, loading, error, refreshStatus } = useApi()

  return (
    <div className="app">
      <header>
        <h1>ProtonVPN Exit Node</h1>
        <p className="subtitle">Tailscale Exit Node Management Dashboard</p>
      </header>

      <main>
        <section className="panel">
          <h2>Exit Node Status</h2>
          <StatusPanel status={status} loading={loading} error={error} />
        </section>

        <section className="panel">
          <h2>Exit Node Controls</h2>
          <ControlButtons onAction={refreshStatus} />
        </section>

        <section className="panel">
          <h2>Configuration</h2>
          <ConfigView />
        </section>

        <section className="panel log-panel">
          <h2>System Logs</h2>
          <LogViewer />
        </section>
      </main>

      <footer>
        <p>Built with React + Vite | Tailscale Exit Node</p>
      </footer>
    </div>
  )
}

export default App
