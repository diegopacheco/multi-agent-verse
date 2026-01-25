import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import TaskList from './TaskList'

const mockTasks = [
  {
    id: '1',
    description: 'Create main app',
    task_type: 'parallel' as const,
    order: undefined,
    assigned_worker: 'worker-1',
    assigned_tester: undefined,
    status: 'inprogress' as const,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  },
  {
    id: '2',
    description: 'Create run.sh',
    task_type: 'sequential' as const,
    order: 1,
    assigned_worker: undefined,
    assigned_tester: undefined,
    status: 'pending' as const,
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  },
]

describe('TaskList', () => {
  it('should render empty state when no tasks', () => {
    render(<TaskList tasks={[]} />)
    expect(screen.getByText(/No tasks yet/)).toBeDefined()
  })

  it('should render task descriptions', () => {
    render(<TaskList tasks={mockTasks} />)
    expect(screen.getByText('Create main app')).toBeDefined()
    expect(screen.getByText('Create run.sh')).toBeDefined()
  })

  it('should render task IDs', () => {
    render(<TaskList tasks={mockTasks} />)
    expect(screen.getByText('Task #1')).toBeDefined()
    expect(screen.getByText('Task #2')).toBeDefined()
  })

  it('should render task types', () => {
    render(<TaskList tasks={mockTasks} />)
    expect(screen.getByText('PARALLEL')).toBeDefined()
    expect(screen.getByText('SEQUENTIAL')).toBeDefined()
  })

  it('should render assigned worker', () => {
    render(<TaskList tasks={mockTasks} />)
    expect(screen.getByText(/Worker: worker-1/)).toBeDefined()
  })

  it('should accept onTaskClick prop', () => {
    const onTaskClick = vi.fn()
    render(<TaskList tasks={mockTasks} onTaskClick={onTaskClick} />)
  })

  it('should render parallel tasks section', () => {
    render(<TaskList tasks={mockTasks} />)
    expect(screen.getByText(/Parallel Tasks/)).toBeDefined()
  })

  it('should render sequential tasks section', () => {
    render(<TaskList tasks={mockTasks} />)
    expect(screen.getByText(/Sequential Tasks/)).toBeDefined()
  })
})
