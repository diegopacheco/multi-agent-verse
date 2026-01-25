import { useState, useEffect, useRef } from 'react'
import { getProjects, startPreview, stopPreview } from '../api/client'

function PreviewPanel() {
  const [projects, setProjects] = useState<string[]>([])
  const [selectedProject, setSelectedProject] = useState<string | null>(null)
  const [previewUrl, setPreviewUrl] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const pendingTimeoutRef = useRef<number | null>(null)
  const activeProjectRef = useRef<string | null>(null)

  useEffect(() => {
    getProjects()
      .then((res) => setProjects(res.projects))
      .catch(() => {})
  }, [])

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

  return (
    <div className="h-full">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-2xl font-bold">Preview</h2>
        <div className="flex gap-2">
          <button
            onClick={handleRefresh}
            className="px-4 py-2 bg-slate-700 hover:bg-slate-600 rounded-lg text-sm font-medium transition-colors"
          >
            Refresh
          </button>
          {previewUrl && (
            <button
              onClick={handleStop}
              className="px-4 py-2 bg-red-600 hover:bg-red-700 rounded-lg text-sm font-medium transition-colors"
            >
              Stop Preview
            </button>
          )}
        </div>
      </div>
      <div className="flex gap-4 h-[calc(100vh-220px)]">
        <div className="w-[15%] min-w-[140px] bg-slate-800 rounded-lg p-4 overflow-y-auto">
          <h3 className="text-sm font-medium text-slate-400 mb-3">Projects</h3>
          {projects.length === 0 && (
            <p className="text-slate-500 text-sm">No projects found in solutions/</p>
          )}
          <ul className="space-y-1">
            {projects.map((project) => (
              <li key={project}>
                <button
                  onClick={() => handleSelectProject(project)}
                  className={`w-full text-left px-4 py-3 rounded-lg transition-colors ${
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
        <div className="flex-1 bg-slate-800 rounded-lg overflow-hidden">
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
  )
}

export default PreviewPanel
