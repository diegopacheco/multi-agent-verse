import { useState, useEffect, useCallback } from 'react'
import { MODEL_OPTIONS } from '../api/client'

interface PromptPanelProps {
  config: {
    model: string
    cliAgent: string
    workerCount: number
    testerCount: number
  }
  onRun: (projectName: string, prompt: string) => void
  onBack: () => void
  isRunning: boolean
}

function PromptPanel({ config, onRun, onBack, isRunning }: PromptPanelProps) {
  const [projectName, setProjectName] = useState('')
  const [prompt, setPrompt] = useState('')
  const [isFullscreen, setIsFullscreen] = useState(false)

  const modelOption = MODEL_OPTIONS.find(
    (o) => o.cli_agent === config.cliAgent && o.model === config.model
  )

  const handleEsc = useCallback((e: KeyboardEvent) => {
    if (e.key === 'Escape' && isFullscreen) {
      setIsFullscreen(false)
    }
  }, [isFullscreen])

  useEffect(() => {
    document.addEventListener('keydown', handleEsc)
    return () => document.removeEventListener('keydown', handleEsc)
  }, [handleEsc])

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (projectName.trim() && prompt.trim()) {
      const enriched =
        prompt.trim() +
        '\n\nmake sure the app works and have a run.sh that run the app, always run on the port 5678 and /index.html. also create a stop.sh that kills the app started by run.sh.'
      onRun(projectName.trim(), enriched)
    }
  }

  if (isFullscreen) {
    return (
      <div className="fixed inset-0 z-50 bg-slate-900 p-6 flex flex-col">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-2xl font-bold">Prompt</h2>
          <button
            onClick={() => setIsFullscreen(false)}
            className="px-4 py-2 bg-slate-700 hover:bg-slate-600 rounded-lg text-sm font-medium transition-colors"
          >
            Exit Fullscreen (ESC)
          </button>
        </div>
        <textarea
          value={prompt}
          onChange={(e) => setPrompt(e.target.value)}
          placeholder="Describe the task for the agents to complete..."
          disabled={isRunning}
          className="flex-1 w-full px-4 py-3 bg-slate-800 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:border-blue-500 resize-none disabled:opacity-50"
        />
      </div>
    )
  }

  return (
    <div className="h-full flex flex-col">
      <h2 className="text-2xl font-bold mb-4 flex-shrink-0">Prompt Input</h2>
      <form onSubmit={handleSubmit} className="flex-1 flex flex-col min-h-0">
        <div className="grid grid-cols-2 gap-6 mb-4 flex-shrink-0">
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Project Name
            </label>
            <input
              type="text"
              value={projectName}
              onChange={(e) => setProjectName(e.target.value)}
              placeholder="my-project"
              disabled={isRunning}
              className="w-full px-4 py-3 bg-slate-800 border border-slate-600 rounded-lg text-white text-lg placeholder-slate-400 focus:outline-none focus:border-blue-500 disabled:opacity-50"
            />
          </div>
          <div className="bg-slate-800 rounded-lg px-4 py-3">
            <div className="text-xs text-slate-400 mb-2">Configuration Summary</div>
            <div className="flex items-center justify-between">
              <div>
                <span className="text-sm text-slate-400">Model: </span>
                <span className="text-lg text-white font-medium">{modelOption?.label || config.model}</span>
              </div>
              <div>
                <span className="text-sm text-slate-400">Workers: </span>
                <span className="text-2xl text-white font-bold">{config.workerCount}</span>
              </div>
              <div>
                <span className="text-sm text-slate-400">Testers: </span>
                <span className="text-2xl text-white font-bold">{config.testerCount}</span>
              </div>
            </div>
          </div>
        </div>
        <div className="flex-1 flex flex-col min-h-0 mb-4">
          <div className="flex items-center justify-between mb-2">
            <label className="text-sm font-medium text-slate-300">Prompt</label>
            <button
              type="button"
              onClick={() => setIsFullscreen(true)}
              className="px-2 py-1 text-xs bg-slate-700 hover:bg-slate-600 rounded transition-colors"
            >
              Fullscreen
            </button>
          </div>
          <textarea
            value={prompt}
            onChange={(e) => setPrompt(e.target.value)}
            placeholder="Describe the task for the agents to complete..."
            disabled={isRunning}
            className="flex-1 w-full px-4 py-3 bg-slate-800 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:border-blue-500 resize-none disabled:opacity-50"
          />
        </div>
        <div className="flex gap-4 flex-shrink-0">
          <button
            type="button"
            onClick={onBack}
            disabled={isRunning}
            className="px-6 py-3 bg-slate-700 hover:bg-slate-600 disabled:opacity-50 rounded-lg font-semibold transition-colors"
          >
            Back
          </button>
          <button
            type="submit"
            disabled={isRunning || !projectName.trim() || !prompt.trim()}
            className="flex-1 px-6 py-3 bg-green-600 hover:bg-green-700 disabled:bg-slate-600 disabled:cursor-not-allowed rounded-lg font-semibold transition-colors text-white text-lg"
          >
            {isRunning ? 'Running...' : 'Run'}
          </button>
        </div>
      </form>
    </div>
  )
}

export default PromptPanel
