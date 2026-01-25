import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import ConfigPanel from './ConfigPanel'

describe('ConfigPanel', () => {
  it('should render configuration title', () => {
    render(<ConfigPanel onNext={vi.fn()} />)
    expect(screen.getByText('Configuration')).toBeDefined()
  })

  it('should render agent/model dropdown', () => {
    render(<ConfigPanel onNext={vi.fn()} />)
    expect(screen.getByText('Agent / Model')).toBeDefined()
  })

  it('should render worker count slider', () => {
    render(<ConfigPanel onNext={vi.fn()} />)
    expect(screen.getByText(/Number of Workers:/)).toBeDefined()
  })

  it('should render tester count slider', () => {
    render(<ConfigPanel onNext={vi.fn()} />)
    expect(screen.getByText(/Number of Testers:/)).toBeDefined()
  })

  it('should render Next button', () => {
    render(<ConfigPanel onNext={vi.fn()} />)
    expect(screen.getByText('Next')).toBeDefined()
  })

  it('should call onNext when Next button is clicked', () => {
    const onNext = vi.fn()
    render(<ConfigPanel onNext={onNext} />)
    fireEvent.click(screen.getByText('Next'))
    expect(onNext).toHaveBeenCalledTimes(1)
  })

  it('should pass config with model, cliAgent, workerCount, testerCount', () => {
    const onNext = vi.fn()
    render(<ConfigPanel onNext={onNext} />)
    fireEvent.click(screen.getByText('Next'))
    expect(onNext).toHaveBeenCalledWith(
      expect.objectContaining({
        model: expect.any(String),
        cliAgent: expect.any(String),
        workerCount: expect.any(Number),
        testerCount: expect.any(Number),
      })
    )
  })
})
