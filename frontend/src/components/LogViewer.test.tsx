import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import LogViewer from './LogViewer'

describe('LogViewer', () => {
  it('should render agent ID', () => {
    render(<LogViewer agentId="worker-1" logs={null} onClose={vi.fn()} />)
    expect(screen.getByText('worker-1')).toBeDefined()
  })

  it('should render loading state when logs is null', () => {
    render(<LogViewer agentId="worker-1" logs={null} onClose={vi.fn()} />)
    expect(screen.getByText('Loading logs...')).toBeDefined()
  })

  it('should render logs content', () => {
    const logs = {
      logs: 'Test log content',
      status: 'done' as const,
      error: undefined,
      duration: 10,
    }
    render(<LogViewer agentId="worker-1" logs={logs} onClose={vi.fn()} />)
    expect(screen.getByText('Test log content')).toBeDefined()
  })

  it('should render status', () => {
    const logs = {
      logs: 'Test',
      status: 'running' as const,
      error: undefined,
      duration: 5,
    }
    render(<LogViewer agentId="worker-1" logs={logs} onClose={vi.fn()} />)
    expect(screen.getByText('Status: RUNNING')).toBeDefined()
  })

  it('should render duration', () => {
    const logs = {
      logs: 'Test',
      status: 'done' as const,
      error: undefined,
      duration: 42,
    }
    render(<LogViewer agentId="worker-1" logs={logs} onClose={vi.fn()} />)
    expect(screen.getByText('Duration: 42s')).toBeDefined()
  })

  it('should render error when present', () => {
    const logs = {
      logs: 'Test',
      status: 'error' as const,
      error: 'Something went wrong',
      duration: 5,
    }
    render(<LogViewer agentId="worker-1" logs={logs} onClose={vi.fn()} />)
    expect(screen.getByText(/Something went wrong/)).toBeDefined()
  })

  it('should call onClose when close button is clicked', () => {
    const onClose = vi.fn()
    render(<LogViewer agentId="worker-1" logs={null} onClose={onClose} />)
    fireEvent.click(screen.getByText('x'))
    expect(onClose).toHaveBeenCalledTimes(1)
  })
})
