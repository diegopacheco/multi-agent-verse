import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import AgentTree from './AgentTree'

describe('AgentTree', () => {
  it('should render task-splitter node', () => {
    render(<AgentTree workerCount={2} testerCount={1} />)
    expect(screen.getByText(/task-splitter/)).toBeDefined()
  })

  it('should render coordinator node', () => {
    render(<AgentTree workerCount={2} testerCount={1} />)
    expect(screen.getByText(/coordinator/)).toBeDefined()
  })

  it('should render correct number of workers', () => {
    render(<AgentTree workerCount={3} testerCount={1} />)
    expect(screen.getByText(/worker-1/)).toBeDefined()
    expect(screen.getByText(/worker-2/)).toBeDefined()
    expect(screen.getByText(/worker-3/)).toBeDefined()
  })

  it('should render correct number of testers', () => {
    render(<AgentTree workerCount={1} testerCount={2} />)
    expect(screen.getByText(/tester-1/)).toBeDefined()
    expect(screen.getByText(/tester-2/)).toBeDefined()
  })

  it('should render with agent info when provided', () => {
    const taskSplitter = {
      id: 'task-splitter',
      role: 'tasksplitter' as const,
      model: 'opus-4-5',
      status: 'done' as const,
      worktree: undefined,
      started_at: undefined,
      finished_at: undefined,
    }
    render(<AgentTree workerCount={1} testerCount={1} taskSplitter={taskSplitter} />)
    expect(screen.getByText(/task-splitter/)).toBeDefined()
  })

  it('should render agent hierarchy title', () => {
    render(<AgentTree workerCount={1} testerCount={1} />)
    expect(screen.getByText('Agent Hierarchy')).toBeDefined()
  })
})
