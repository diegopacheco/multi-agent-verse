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

  return (
    <div className="h-full flex flex-col">
      <h2 className="text-2xl font-bold mb-4">Execution Monitor</h2>
      <ProgressBar
        progress={status?.progress || 0}
        elapsedTime={status?.elapsed_time || 0}
      />
      <div className="flex-1 grid grid-cols-2 gap-4 mt-4 min-h-0">
        <div className="flex flex-col min-h-0">
          <h3 className="text-lg font-semibold mb-2 text-slate-300">Agents</h3>
          <div className="flex-1 overflow-auto">
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
        <div className="flex flex-col min-h-0">
          <h3 className="text-lg font-semibold mb-2 text-slate-300">Tasks</h3>
          <div className="flex-1 overflow-auto bg-slate-800 rounded-lg p-4">
            <TaskList tasks={tasks} />
          </div>
        </div>
      </div>
      <div className="mt-4">
        <h3 className="text-lg font-semibold mb-2 text-slate-300">Event Log</h3>
        <div className="bg-slate-800 rounded-lg p-4">
          <EventLog events={events} />
        </div>
      </div>
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
