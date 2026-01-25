import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import PromptPanel from './PromptPanel'

const mockConfig = {
  model: 'opus-4-5',
  cliAgent: 'claude-code',
  workerCount: 3,
  testerCount: 2,
}

describe('PromptPanel', () => {
  it('should render prompt input title', () => {
    render(<PromptPanel config={mockConfig} onRun={vi.fn()} onBack={vi.fn()} isRunning={false} />)
    expect(screen.getByText('Prompt Input')).toBeDefined()
  })

  it('should render configuration summary', () => {
    render(<PromptPanel config={mockConfig} onRun={vi.fn()} onBack={vi.fn()} isRunning={false} />)
    expect(screen.getByText('Configuration Summary')).toBeDefined()
  })

  it('should render project name input', () => {
    render(<PromptPanel config={mockConfig} onRun={vi.fn()} onBack={vi.fn()} isRunning={false} />)
    expect(screen.getByText('Project Name')).toBeDefined()
  })

  it('should render prompt textarea', () => {
    render(<PromptPanel config={mockConfig} onRun={vi.fn()} onBack={vi.fn()} isRunning={false} />)
    expect(screen.getByText('Prompt')).toBeDefined()
  })

  it('should render Back button', () => {
    render(<PromptPanel config={mockConfig} onRun={vi.fn()} onBack={vi.fn()} isRunning={false} />)
    expect(screen.getByText('Back')).toBeDefined()
  })

  it('should render Run button', () => {
    render(<PromptPanel config={mockConfig} onRun={vi.fn()} onBack={vi.fn()} isRunning={false} />)
    expect(screen.getByText('Run')).toBeDefined()
  })

  it('should call onBack when Back button is clicked', () => {
    const onBack = vi.fn()
    render(<PromptPanel config={mockConfig} onRun={vi.fn()} onBack={onBack} isRunning={false} />)
    fireEvent.click(screen.getByText('Back'))
    expect(onBack).toHaveBeenCalledTimes(1)
  })

  it('should show Running... when isRunning is true', () => {
    render(<PromptPanel config={mockConfig} onRun={vi.fn()} onBack={vi.fn()} isRunning={true} />)
    expect(screen.getByText('Running...')).toBeDefined()
  })

  it('should display worker count in summary', () => {
    render(<PromptPanel config={mockConfig} onRun={vi.fn()} onBack={vi.fn()} isRunning={false} />)
    expect(screen.getByText('3')).toBeDefined()
  })

  it('should display tester count in summary', () => {
    render(<PromptPanel config={mockConfig} onRun={vi.fn()} onBack={vi.fn()} isRunning={false} />)
    expect(screen.getByText('2')).toBeDefined()
  })
})
