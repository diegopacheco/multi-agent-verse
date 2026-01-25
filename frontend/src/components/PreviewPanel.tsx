import { useState, useEffect, useRef, useCallback } from 'react'
import { getProjects, startPreview, stopPreview } from '../api/client'

function PreviewPanel() {
  const [projects, setProjects] = useState<string[]>([])
  const [selectedProject, setSelectedProject] = useState<string | null>(null)
  const [previewUrl, setPreviewUrl] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [isFullscreen, setIsFullscreen] = useState(false)
  const [searchQuery, setSearchQuery] = useState('')
  const pendingTimeoutRef = useRef<number | null>(null)
  const activeProjectRef = useRef<string | null>(null)

  const handleEscKey = useCallback((event: KeyboardEvent) => {
    if (event.key === 'Escape' && isFullscreen) {
      setIsFullscreen(false)
    }
  }, [isFullscreen])

  useEffect(() => {
    document.addEventListener('keydown', handleEscKey)
    return () => document.removeEventListener('keydown', handleEscKey)
  }, [handleEscKey])

  useEffect(() => {
    getProjects()
      .then((res) => setProjects(res.projects))
      .catch(() => {})
  }, [])

  const filteredProjects = projects.filter((p) =>
    p.toLowerCase().includes(searchQuery.toLowerCase())
  )

  const handleSelectProject = async (project: string) => {
    if (pendingTimeoutRef.current !== null) {
      clearTimeout(pendingTimeoutRef.current)
      pendingTimeoutRef.current = null
    }
    activeProjectRef.current = project
    setLoading(true)
    setSelectedProject(project)
    setPreviewUrl(null)
    setError(null)
    try {
      await stopPreview().catch(() => {})
      if (activeProjectRef.current !== project) {
        return
      }
      const res = await startPreview(project)
      if (activeProjectRef.current !== project) {
        return
      }
      if (res.ok && res.url) {
        const url = res.url
        pendingTimeoutRef.current = window.setTimeout(() => {
          pendingTimeoutRef.current = null
          if (activeProjectRef.current === project) {
            setPreviewUrl(url + '?t=' + Date.now())
            setLoading(false)
          }
        }, 2000)
      } else {
        if (activeProjectRef.current === project) {
          setError('Failed to start preview - run.sh may be missing')
          setLoading(false)
        }
      }
    } catch {
      if (activeProjectRef.current === project) {
        setError('Failed to start preview - run.sh may be missing')
        setLoading(false)
      }
    }
  }

  const handleStop = async () => {
    if (pendingTimeoutRef.current !== null) {
      clearTimeout(pendingTimeoutRef.current)
      pendingTimeoutRef.current = null
    }
    activeProjectRef.current = null
    setLoading(false)
    setPreviewUrl(null)
    setSelectedProject(null)
    setError(null)
    try {
      await stopPreview()
    } catch {}
  }

  useEffect(() => {
    return () => {
      if (pendingTimeoutRef.current !== null) {
        clearTimeout(pendingTimeoutRef.current)
      }
    }
  }, [])

  const handleRefresh = () => {
    getProjects()
      .then((res) => setProjects(res.projects))
      .catch(() => {})
  }

  if (isFullscreen && previewUrl) {
    return (
      <div className="fixed inset-0 z-50 bg-black">
        <div className="absolute top-4 right-4 z-10 flex gap-2">
          <button
            onClick={() => setIsFullscreen(false)}
            className="px-4 py-2 bg-slate-700 hover:bg-slate-600 rounded-lg text-sm font-medium transition-colors"
          >
            Exit Fullscreen (ESC)
          </button>
          <button
            onClick={() => {
              setIsFullscreen(false)
              handleStop()
            }}
            className="px-4 py-2 bg-red-600 hover:bg-red-700 rounded-lg text-sm font-medium transition-colors"
          >
            Stop
          </button>
        </div>
        <iframe
          src={previewUrl}
          className="w-full h-full border-0"
          title="Solution Preview Fullscreen"
        />
      </div>
    )
  }

  return (
    <div className="h-full flex flex-col">
      <div className="flex items-center justify-between mb-4 flex-shrink-0">
        <h2 className="text-2xl font-bold">Preview</h2>
        <div className="flex gap-2">
          <button
            onClick={handleRefresh}
            className="px-4 py-2 bg-slate-700 hover:bg-slate-600 rounded-lg text-sm font-medium transition-colors"
          >
            Refresh
          </button>
          {previewUrl && (
            <>
              <button
                onClick={() => setIsFullscreen(true)}
                className="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg text-sm font-medium transition-colors"
              >
                Fullscreen
              </button>
              <button
                onClick={handleStop}
                className="px-4 py-2 bg-red-600 hover:bg-red-700 rounded-lg text-sm font-medium transition-colors"
              >
                Stop Preview
              </button>
            </>
          )}
        </div>
      </div>
      <div className="flex gap-4 flex-1 min-h-0">
        <div className="w-64 bg-slate-800 rounded-lg p-4 flex flex-col">
          <h3 className="text-sm font-medium text-slate-400 mb-2 flex-shrink-0">Projects</h3>
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search projects..."
            className="w-full px-3 py-2 mb-3 bg-slate-700 border border-slate-600 rounded-lg text-white text-sm placeholder-slate-400 focus:outline-none focus:border-blue-500 flex-shrink-0"
          />
          <div className="flex-1 overflow-y-auto">
            {filteredProjects.length === 0 && (
              <p className="text-slate-500 text-sm">No projects found</p>
            )}
            <ul className="space-y-1">
              {filteredProjects.map((project) => (
                <li key={project}>
                  <button
                    onClick={() => handleSelectProject(project)}
                    className={`w-full text-left px-3 py-2 rounded-lg transition-colors text-sm ${
                      selectedProject === project
                        ? 'bg-blue-600 text-white'
                        : 'bg-slate-700 hover:bg-slate-600 text-slate-200'
                    }`}
                  >
                    {project}
                  </button>
                </li>
              ))}
            </ul>
          </div>
        </div>
        <div className="flex-1 bg-slate-800 rounded-lg overflow-hidden flex flex-col">
          <div className="flex items-center justify-end p-2 border-b border-slate-700 flex-shrink-0">
            {previewUrl && (
              <button
                onClick={() => setIsFullscreen(true)}
                className="px-2 py-1 text-xs bg-slate-700 hover:bg-slate-600 rounded transition-colors"
              >
                Fullscreen
              </button>
            )}
          </div>
          <div className="flex-1">
            {loading && (
              <div className="flex items-center justify-center h-full">
                <p className="text-slate-400">Starting project...</p>
              </div>
            )}
            {!loading && previewUrl && (
              <iframe
                key={previewUrl}
                src={previewUrl}
                className="w-full h-full border-0"
                title="Solution Preview"
              />
            )}
            {!loading && !previewUrl && error && (
              <div className="flex items-center justify-center h-full">
                <p className="text-red-400">{error}</p>
              </div>
            )}
            {!loading && !previewUrl && !error && (
              <div className="flex items-center justify-center h-full">
                <p className="text-slate-500">Select a project to preview</p>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}

export default PreviewPanel
