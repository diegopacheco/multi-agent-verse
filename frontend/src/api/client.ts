export type AgentStatus = 'pending' | 'running' | 'done' | 'error' | 'timeout'
export type AgentRole = 'tasksplitter' | 'coordinator' | 'worker' | 'tester'
export type TaskStatus = 'pending' | 'inprogress' | 'testing' | 'done' | 'failed'
export type TaskType = 'parallel' | 'sequential'

export interface AgentInfo {
  id: string
  role: AgentRole
  model: string
  status: AgentStatus
  worktree?: string
  started_at?: string
  finished_at?: string
}

export interface Task {
  id: string
  description: string
  task_type: TaskType
  order?: number
  assigned_worker?: string
  assigned_tester?: string
  status: TaskStatus
  created_at: string
  updated_at: string
}

export interface Event {
  timestamp: string
  level: string
  agent_id?: string
  message: string
}

export interface CreateSessionRequest {
  model: string
  cli_agent: string
  worker_count: number
  tester_count: number
}

export interface CreateSessionResponse {
  session_id: string
}

export interface RunRequest {
  prompt: string
  project_name: string
}

export interface RunResponse {
  ok: boolean
}

export interface StatusResponse {
  task_splitter: AgentInfo
  coordinator: AgentInfo
  workers: AgentInfo[]
  testers: AgentInfo[]
  progress: number
  elapsed_time: number
}

export interface LogsResponse {
  logs: string
  status: AgentStatus
  error?: string
  duration: number
}

export interface TasksResponse {
  tasks: Task[]
}

export interface EventsResponse {
  events: Event[]
}

export interface ProjectsResponse {
  projects: string[]
}

export interface PreviewResponse {
  ok: boolean
  url?: string
}

const BASE_URL = '/api'

export async function createSession(req: CreateSessionRequest): Promise<CreateSessionResponse> {
  const response = await fetch(`${BASE_URL}/session`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(req),
  })
  if (!response.ok) throw new Error('Failed to create session')
  return response.json()
}

export async function runSession(sessionId: string, req: RunRequest): Promise<RunResponse> {
  const response = await fetch(`${BASE_URL}/run/${sessionId}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(req),
  })
  if (!response.ok) throw new Error('Failed to run session')
  return response.json()
}

export async function getStatus(sessionId: string): Promise<StatusResponse> {
  const response = await fetch(`${BASE_URL}/status/${sessionId}`)
  if (!response.ok) throw new Error('Failed to get status')
  return response.json()
}

export async function getLogs(sessionId: string, agentId: string): Promise<LogsResponse> {
  const response = await fetch(`${BASE_URL}/logs/${sessionId}/${agentId}`)
  if (!response.ok) throw new Error('Failed to get logs')
  return response.json()
}

export async function getTasks(sessionId: string): Promise<TasksResponse> {
  const response = await fetch(`${BASE_URL}/tasks/${sessionId}`)
  if (!response.ok) throw new Error('Failed to get tasks')
  return response.json()
}

export async function getEvents(sessionId: string): Promise<EventsResponse> {
  const response = await fetch(`${BASE_URL}/events/${sessionId}`)
  if (!response.ok) throw new Error('Failed to get events')
  return response.json()
}

export async function getProjects(): Promise<ProjectsResponse> {
  const response = await fetch(`${BASE_URL}/projects`)
  if (!response.ok) throw new Error('Failed to get projects')
  return response.json()
}

export async function startPreview(projectName: string): Promise<PreviewResponse> {
  const response = await fetch(`${BASE_URL}/preview/start/${projectName}`, {
    method: 'POST',
  })
  if (!response.ok) throw new Error('Failed to start preview')
  return response.json()
}

export async function stopPreview(): Promise<PreviewResponse> {
  const response = await fetch(`${BASE_URL}/preview/stop`, {
    method: 'POST',
  })
  if (!response.ok) throw new Error('Failed to stop preview')
  return response.json()
}

export const MODEL_OPTIONS = [
  { cli_agent: 'claude-code', model: 'claude-opus-4-5-20251101', label: 'Claude Code / Opus 4.5' },
  { cli_agent: 'codex', model: 'gpt-5.2', label: 'Codex / GPT 5.2' },
  { cli_agent: 'copilot', model: 'claude-sonnet-4', label: 'Copilot / Claude Sonnet 4' },
  { cli_agent: 'gemini', model: 'gemini-3', label: 'Gemini / Gemini 3' },
  { cli_agent: 'llr3', model: 'llama3', label: 'LLR3 / Llama 3' },
]
