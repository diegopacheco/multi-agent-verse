import { AgentStatus, LogsResponse } from '../api/client'

interface LogViewerProps {
  agentId: string
  logs: LogsResponse | null
  onClose: () => void
}

function getStatusColor(status: AgentStatus): string {
  switch (status) {
    case 'done':
      return 'text-green-400'
    case 'running':
      return 'text-blue-400'
    case 'error':
      return 'text-red-400'
    case 'timeout':
      return 'text-yellow-400'
    default:
      return 'text-slate-400'
  }
}

function LogViewer({ agentId, logs, onClose }: LogViewerProps) {
  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-slate-800 rounded-lg w-full max-w-3xl max-h-[80vh] flex flex-col">
        <div className="flex justify-between items-center p-4 border-b border-slate-700">
          <div>
            <h3 className="text-lg font-semibold">{agentId}</h3>
            {logs && (
              <div className="flex gap-4 text-sm mt-1">
                <span className={getStatusColor(logs.status)}>
                  Status: {logs.status.toUpperCase()}
                </span>
                <span className="text-slate-400">
                  Duration: {logs.duration}s
                </span>
              </div>
            )}
          </div>
          <button
            onClick={onClose}
            className="text-slate-400 hover:text-white text-2xl"
          >
            x
          </button>
        </div>
        <div className="flex-1 overflow-auto p-4">
          {logs?.error && (
            <div className="bg-red-900/30 border border-red-700 rounded p-3 mb-4 text-red-400">
              Error: {logs.error}
            </div>
          )}
          <pre className="font-mono text-sm text-slate-300 whitespace-pre-wrap">
            {logs?.logs || 'Loading logs...'}
          </pre>
        </div>
      </div>
    </div>
  )
}

export default LogViewer
