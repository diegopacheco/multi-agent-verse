import { useState, useEffect } from 'react'
import { getProjects, startPreview, stopPreview } from '../api/client'

function PreviewPanel() {
  const [projects, setProjects] = useState<string[]>([])
  const [selectedProject, setSelectedProject] = useState<string | null>(null)
  const [previewUrl, setPreviewUrl] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    getProjects()
      .then((res) => setProjects(res.projects))
      .catch(() => {})
  }, [])

  const handleSelectProject = async (project: string) => {
    setLoading(true)
    setSelectedProject(project)
    setPreviewUrl(null)
    try {
      if (previewUrl) {
        await stopPreview()
      }
      const res = await startPreview(project)
      if (res.ok && res.url) {
        setTimeout(() => {
          setPreviewUrl(res.url!)
          setLoading(false)
        }, 2000)
      } else {
        setLoading(false)
      }
    } catch {
      setLoading(false)
    }
  }

  const handleStop = async () => {
    try {
      await stopPreview()
    } catch {}
    setPreviewUrl(null)
    setSelectedProject(null)
  }

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
        <div className="w-1/2 bg-slate-800 rounded-lg p-4 overflow-y-auto">
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
        <div className="w-1/2 bg-slate-800 rounded-lg overflow-hidden">
          {loading && (
            <div className="flex items-center justify-center h-full">
              <p className="text-slate-400">Starting project...</p>
            </div>
          )}
          {!loading && previewUrl && (
            <iframe
              src={previewUrl}
              className="w-full h-full border-0"
              title="Solution Preview"
            />
          )}
          {!loading && !previewUrl && (
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
