import { AgentInfo, AgentStatus } from '../api/client'

interface AgentTreeProps {
  taskSplitter?: AgentInfo
  coordinator?: AgentInfo
  workers?: AgentInfo[]
  testers?: AgentInfo[]
  workerCount: number
  testerCount: number
  onAgentClick?: (agentId: string) => void
  selectedAgent?: string
}

function getStatusColor(status?: AgentStatus): string {
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

function getStatusBadge(status?: AgentStatus): string {
  switch (status) {
    case 'done':
      return '[DONE]'
    case 'running':
      return '[RUNNING]'
    case 'error':
      return '[ERROR]'
    case 'timeout':
      return '[TIMEOUT]'
    default:
      return '[PENDING]'
  }
}

function AgentTree({
  taskSplitter,
  coordinator,
  workers,
  testers,
  workerCount,
  testerCount,
  onAgentClick,
  selectedAgent,
}: AgentTreeProps) {
  const renderAgent = (
    id: string,
    label: string,
    status?: AgentStatus,
    indent: number = 0
  ) => {
    const isClickable = !!onAgentClick
    const isSelected = selectedAgent === id
    const padding = indent * 24
    return (
      <div
        key={id}
        className={`py-1 font-mono text-sm cursor-pointer hover:bg-slate-700/50 rounded px-2 ${
          isSelected ? 'bg-slate-700' : ''
        }`}
        style={{ paddingLeft: `${padding + 8}px` }}
        onClick={() => isClickable && onAgentClick(id)}
      >
        <span className={getStatusColor(status)}>
          {indent > 0 && (
            <span className="text-slate-600">
              {indent === 1 ? '└── ' : '    ├── '}
            </span>
          )}
          {label}{' '}
          <span className="text-xs">{getStatusBadge(status)}</span>
        </span>
      </div>
    )
  }

  const actualWorkers = workers || []
  const actualTesters = testers || []

  return (
    <div className="bg-slate-800 rounded-lg p-4">
      <h3 className="text-lg font-semibold mb-4 text-slate-200">Agent Hierarchy</h3>
      <div className="space-y-1">
        {renderAgent(
          'task-splitter',
          'task-splitter (1)',
          taskSplitter?.status,
          0
        )}
        {renderAgent(
          'coordinator',
          'coordinator (1)',
          coordinator?.status,
          1
        )}
        {actualWorkers.length > 0 ? (
          actualWorkers.map((w) =>
            renderAgent(w.id, w.id, w.status, 2)
          )
        ) : (
          Array.from({ length: workerCount }, (_, i) =>
            renderAgent(`worker-${i + 1}`, `worker-${i + 1}`, undefined, 2)
          )
        )}
        {actualTesters.length > 0 ? (
          actualTesters.map((t) =>
            renderAgent(t.id, t.id, t.status, 2)
          )
        ) : (
          Array.from({ length: testerCount }, (_, i) =>
            renderAgent(`tester-${i + 1}`, `tester-${i + 1}`, undefined, 2)
          )
        )}
      </div>
    </div>
  )
}

export default AgentTree
