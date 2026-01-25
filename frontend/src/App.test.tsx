import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import App from './App'

vi.mock('./api/client', () => ({
  MODEL_OPTIONS: [
    { cli_agent: 'claude-code', model: 'opus-4-5', label: 'claude-code / opus-4-5' },
  ],
  createSession: vi.fn().mockResolvedValue({ session_id: 'test-session' }),
  runSession: vi.fn().mockResolvedValue({ ok: true }),
  getStatus: vi.fn().mockResolvedValue({ task_splitter: null, coordinator: null, workers: [], testers: [], progress: 0, elapsed_time: 0 }),
  getTasks: vi.fn().mockResolvedValue({ tasks: [] }),
  getEvents: vi.fn().mockResolvedValue({ events: [] }),
  getProjects: vi.fn().mockResolvedValue({ projects: [] }),
}))

describe('App', () => {
  it('should render app title', () => {
    render(<App />)
    expect(screen.getByText('Multi-Agent Verse')).toBeDefined()
  })

  it('should render app subtitle', () => {
    render(<App />)
    expect(screen.getByText(/Multi-agent orchestrator/)).toBeDefined()
  })

  it('should render Configuration tab', () => {
    render(<App />)
    expect(screen.getByText('1. Configuration')).toBeDefined()
  })

  it('should render Prompt tab', () => {
    render(<App />)
    expect(screen.getByText('2. Prompt')).toBeDefined()
  })

  it('should render Monitor tab', () => {
    render(<App />)
    expect(screen.getByText('3. Monitor')).toBeDefined()
  })

  it('should render Preview tab', () => {
    render(<App />)
    expect(screen.getByText('4. Preview')).toBeDefined()
  })

  it('should show Configuration panel by default', () => {
    render(<App />)
    expect(screen.getByText('Configuration')).toBeDefined()
  })
})
