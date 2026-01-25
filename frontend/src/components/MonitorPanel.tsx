import { useState, useEffect, useCallback } from 'react'
import {
  StatusResponse,
  Task,
  Event,
  LogsResponse,
  getStatus,
  getTasks,
  getEvents,
  getLogs,
} from '../api/client'
import AgentTree from './AgentTree'
import TaskList from './TaskList'
import EventLog from './EventLog'
import ProgressBar from './ProgressBar'
import LogViewer from './LogViewer'

type FullscreenPanel = 'agents' | 'tasks' | 'events' | null

interface MonitorPanelProps {
  sessionId: string
  config: {
    workerCount: number
    testerCount: number
  }
}

function MonitorPanel({ sessionId, config }: MonitorPanelProps) {
  const [status, setStatus] = useState<StatusResponse | null>(null)
  const [tasks, setTasks] = useState<Task[]>([])
  const [events, setEvents] = useState<Event[]>([])
  const [selectedAgent, setSelectedAgent] = useState<string | null>(null)
  const [agentLogs, setAgentLogs] = useState<LogsResponse | null>(null)
  const [fullscreenPanel, setFullscreenPanel] = useState<FullscreenPanel>(null)

  useEffect(() => {
    const handleEsc = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && fullscreenPanel) {
        setFullscreenPanel(null)
      }
    }
    document.addEventListener('keydown', handleEsc)
    return () => document.removeEventListener('keydown', handleEsc)
  }, [fullscreenPanel])

  const pollData = useCallback(async () => {
    try {
      const [statusRes, tasksRes, eventsRes] = await Promise.all([
        getStatus(sessionId),
        getTasks(sessionId),
        getEvents(sessionId),
      ])
      setStatus(statusRes)
      setTasks(tasksRes.tasks)
      setEvents(eventsRes.events)
    } catch (error) {
      console.error('Failed to poll data:', error)
    }
  }, [sessionId])

  useEffect(() => {
    pollData()
    const interval = setInterval(pollData, 2000)
    return () => clearInterval(interval)
  }, [pollData])

  const handleAgentClick = async (agentId: string) => {
    setSelectedAgent(agentId)
    try {
      const logs = await getLogs(sessionId, agentId)
      setAgentLogs(logs)
    } catch (error) {
      setAgentLogs(null)
    }
  }

  const handleCloseModal = () => {
    setSelectedAgent(null)
    setAgentLogs(null)
  }

  const handleTaskClick = async (task: Task) => {
    if (task.assigned_worker) {
      setSelectedAgent(task.assigned_worker)
      try {
        const logs = await getLogs(sessionId, task.assigned_worker)
        setAgentLogs(logs)
      } catch (error) {
        setAgentLogs(null)
      }
    }
  }

  return (
    <div className="h-full flex flex-col overflow-hidden">
      <div className="flex-shrink-0">
        <h2 className="text-2xl font-bold mb-4">Execution Monitor</h2>
        <ProgressBar
          progress={status?.progress || 0}
          elapsedTime={status?.elapsed_time || 0}
        />
      </div>
      <div className="flex-1 grid grid-cols-2 gap-4 mt-4 min-h-0 overflow-hidden">
        <div className="flex flex-col min-h-0 overflow-hidden">
          <div className="flex items-center justify-between mb-2 flex-shrink-0">
            <h3 className="text-lg font-semibold text-slate-300">Agents</h3>
            <button
              onClick={() => setFullscreenPanel('agents')}
              className="px-2 py-1 text-xs bg-slate-700 hover:bg-slate-600 rounded transition-colors"
            >
              Fullscreen
            </button>
          </div>
          <div className="flex-1 overflow-auto bg-slate-800 rounded-lg p-4">
            <AgentTree
              taskSplitter={status?.task_splitter}
              coordinator={status?.coordinator}
              workers={status?.workers}
              testers={status?.testers}
              workerCount={config.workerCount}
              testerCount={config.testerCount}
              onAgentClick={handleAgentClick}
              selectedAgent={selectedAgent || undefined}
            />
          </div>
        </div>
        <div className="flex flex-col min-h-0 overflow-hidden">
          <div className="flex items-center justify-between mb-2 flex-shrink-0">
            <h3 className="text-lg font-semibold text-slate-300">Tasks</h3>
            <button
              onClick={() => setFullscreenPanel('tasks')}
              className="px-2 py-1 text-xs bg-slate-700 hover:bg-slate-600 rounded transition-colors"
            >
              Fullscreen
            </button>
          </div>
          <div className="flex-1 overflow-auto bg-slate-800 rounded-lg p-4">
            <TaskList tasks={tasks} onTaskClick={handleTaskClick} />
          </div>
        </div>
      </div>
      <div className="flex-shrink-0 mt-4 max-h-48 flex flex-col">
        <div className="flex items-center justify-between mb-2">
          <h3 className="text-lg font-semibold text-slate-300">Event Log</h3>
          <button
            onClick={() => setFullscreenPanel('events')}
            className="px-2 py-1 text-xs bg-slate-700 hover:bg-slate-600 rounded transition-colors"
          >
            Fullscreen
          </button>
        </div>
        <div className="flex-1 overflow-auto bg-slate-800 rounded-lg p-4">
          <EventLog events={events} />
        </div>
      </div>
      {fullscreenPanel === 'agents' && (
        <div className="fixed inset-0 z-50 bg-slate-900 p-6 flex flex-col">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-2xl font-bold">Agents</h2>
            <button
              onClick={() => setFullscreenPanel(null)}
              className="px-4 py-2 bg-slate-700 hover:bg-slate-600 rounded-lg text-sm font-medium transition-colors"
            >
              Exit Fullscreen (ESC)
            </button>
          </div>
          <div className="flex-1 overflow-auto bg-slate-800 rounded-lg p-6">
            <AgentTree
              taskSplitter={status?.task_splitter}
              coordinator={status?.coordinator}
              workers={status?.workers}
              testers={status?.testers}
              workerCount={config.workerCount}
              testerCount={config.testerCount}
              onAgentClick={handleAgentClick}
              selectedAgent={selectedAgent || undefined}
            />
          </div>
        </div>
      )}
      {fullscreenPanel === 'tasks' && (
        <div className="fixed inset-0 z-50 bg-slate-900 p-6 flex flex-col">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-2xl font-bold">Tasks</h2>
            <button
              onClick={() => setFullscreenPanel(null)}
              className="px-4 py-2 bg-slate-700 hover:bg-slate-600 rounded-lg text-sm font-medium transition-colors"
            >
              Exit Fullscreen (ESC)
            </button>
          </div>
          <div className="flex-1 overflow-auto bg-slate-800 rounded-lg p-6">
            <TaskList tasks={tasks} onTaskClick={handleTaskClick} />
          </div>
        </div>
      )}
      {fullscreenPanel === 'events' && (
        <div className="fixed inset-0 z-50 bg-slate-900 p-6 flex flex-col">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-2xl font-bold">Event Log</h2>
            <button
              onClick={() => setFullscreenPanel(null)}
              className="px-4 py-2 bg-slate-700 hover:bg-slate-600 rounded-lg text-sm font-medium transition-colors"
            >
              Exit Fullscreen (ESC)
            </button>
          </div>
          <div className="flex-1 overflow-auto bg-slate-800 rounded-lg p-6">
            <EventLog events={events} />
          </div>
        </div>
      )}
      {selectedAgent && (
        <LogViewer
          agentId={selectedAgent}
          logs={agentLogs}
          onClose={handleCloseModal}
        />
      )}
    </div>
  )
}

export default MonitorPanel
