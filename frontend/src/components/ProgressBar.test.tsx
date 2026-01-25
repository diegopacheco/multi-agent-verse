import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import ProgressBar from './ProgressBar'

describe('ProgressBar', () => {
  it('should render progress percentage with one decimal', () => {
    render(<ProgressBar progress={50} elapsedTime={0} />)
    expect(screen.getByText(/50\.0%/)).toBeDefined()
  })

  it('should render 0.0% for zero progress', () => {
    render(<ProgressBar progress={0} elapsedTime={0} />)
    expect(screen.getByText(/0\.0%/)).toBeDefined()
  })

  it('should render 100.0% for complete progress', () => {
    render(<ProgressBar progress={100} elapsedTime={0} />)
    expect(screen.getByText(/100\.0%/)).toBeDefined()
  })

  it('should show decimal for fractional progress', () => {
    render(<ProgressBar progress={33.33} elapsedTime={0} />)
    expect(screen.getByText(/33\.3%/)).toBeDefined()
  })

  it('should display elapsed time in MM:SS format', () => {
    render(<ProgressBar progress={0} elapsedTime={45} />)
    expect(screen.getByText(/0:45/)).toBeDefined()
  })

  it('should display elapsed time with minutes', () => {
    render(<ProgressBar progress={0} elapsedTime={125} />)
    expect(screen.getByText(/2:05/)).toBeDefined()
  })

  it('should pad seconds with zero', () => {
    render(<ProgressBar progress={0} elapsedTime={65} />)
    expect(screen.getByText(/1:05/)).toBeDefined()
  })
})
