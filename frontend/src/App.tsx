import { StatusPanel } from './components/StatusPanel'
import { LogViewer } from './components/LogViewer'
import { ControlButtons } from './components/ControlButtons'
import { ConfigView } from './components/ConfigView'
import { useApi } from './hooks/useApi'

function App() {
  const { status, loading, error, refreshStatus } = useApi()

  return (
    <div className="app">
      <header>
        <h1>ProtonVPN over Tailscale</h1>
        <p className="subtitle">VPN Management Dashboard</p>
      </header>

      <main>
        <section className="panel">
          <h2>Connection Status</h2>
          <StatusPanel status={status} loading={loading} error={error} />
        </section>

        <section className="panel">
          <h2>Controls</h2>
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
        <p>Built with React + Vite</p>
      </footer>
    </div>
  )
}

export default App
