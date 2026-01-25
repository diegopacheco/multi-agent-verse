import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import MonitorPanel from './MonitorPanel'

vi.mock('../api/client', () => ({
  getStatus: vi.fn().mockResolvedValue({ 
    task_splitter: null, 
    coordinator: null, 
    workers: [], 
    testers: [], 
    progress: 0, 
    elapsed_time: 0 
  }),
  getTasks: vi.fn().mockResolvedValue({ tasks: [] }),
  getEvents: vi.fn().mockResolvedValue({ events: [] }),
  getLogs: vi.fn().mockResolvedValue({ logs: '', status: 'pending', error: null, duration: 0 }),
}))

describe('MonitorPanel', () => {
  const mockConfig = {
    workerCount: 2,
    testerCount: 1,
  }

  it('should render execution monitor title', () => {
    render(<MonitorPanel sessionId="test-session" config={mockConfig} />)
    expect(screen.getByText('Execution Monitor')).toBeDefined()
  })

  it('should render Agents section', () => {
    render(<MonitorPanel sessionId="test-session" config={mockConfig} />)
    expect(screen.getByText('Agents')).toBeDefined()
  })

  it('should render Tasks section', () => {
    render(<MonitorPanel sessionId="test-session" config={mockConfig} />)
    expect(screen.getByText('Tasks')).toBeDefined()
  })

  it('should render Event Log section', () => {
    render(<MonitorPanel sessionId="test-session" config={mockConfig} />)
    expect(screen.getByText('Event Log')).toBeDefined()
  })

  it('should render fullscreen buttons', () => {
    render(<MonitorPanel sessionId="test-session" config={mockConfig} />)
    const fullscreenButtons = screen.getAllByText('Fullscreen')
    expect(fullscreenButtons.length).toBe(3)
  })
})
