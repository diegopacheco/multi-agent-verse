import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import PreviewPanel from './PreviewPanel'

vi.mock('../api/client', () => ({
  getProjects: vi.fn().mockResolvedValue({ projects: ['project1', 'project2'] }),
  startPreview: vi.fn().mockResolvedValue({ ok: true, url: 'http://localhost:5678/index.html' }),
  stopPreview: vi.fn().mockResolvedValue({ ok: true }),
}))

describe('PreviewPanel', () => {
  it('should render preview title', () => {
    render(<PreviewPanel />)
    expect(screen.getByText('Preview')).toBeDefined()
  })

  it('should render projects section', () => {
    render(<PreviewPanel />)
    expect(screen.getByText('Projects')).toBeDefined()
  })

  it('should render refresh button', () => {
    render(<PreviewPanel />)
    expect(screen.getByText('Refresh')).toBeDefined()
  })

  it('should render placeholder when no project selected', () => {
    render(<PreviewPanel />)
    expect(screen.getByText('Select a project to preview')).toBeDefined()
  })
})
