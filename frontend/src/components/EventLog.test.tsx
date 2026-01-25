import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import EventLog from './EventLog'

const mockEvents = [
  {
    timestamp: '2024-01-01T10:00:00Z',
    level: 'INFO',
    agent_id: 'task-splitter',
    message: 'task-splitter started',
  },
  {
    timestamp: '2024-01-01T10:01:00Z',
    level: 'ERROR',
    agent_id: 'worker-1',
    message: 'Worker failed',
  },
]

describe('EventLog', () => {
  it('should render empty state when no events', () => {
    render(<EventLog events={[]} />)
    expect(screen.getByText(/No events yet/)).toBeDefined()
  })

  it('should render event messages', () => {
    render(<EventLog events={mockEvents} />)
    expect(screen.getByText(/task-splitter started/)).toBeDefined()
    expect(screen.getByText(/Worker failed/)).toBeDefined()
  })

  it('should render event levels', () => {
    render(<EventLog events={mockEvents} />)
    expect(screen.getByText('[INFO]')).toBeDefined()
    expect(screen.getByText('[ERROR]')).toBeDefined()
  })

  it('should render agent IDs', () => {
    render(<EventLog events={mockEvents} />)
    expect(screen.getByText(/task-splitter:/)).toBeDefined()
    expect(screen.getByText(/worker-1:/)).toBeDefined()
  })
})
