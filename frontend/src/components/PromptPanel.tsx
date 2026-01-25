import { useState } from 'react'
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

  const modelOption = MODEL_OPTIONS.find(
    (o) => o.cli_agent === config.cliAgent && o.model === config.model
  )

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (projectName.trim() && prompt.trim()) {
      const enriched =
        prompt.trim() +
        '\n\nmake sure the app works and have a run.sh that run the app, always run on the port 5678 and /index.html. also create a stop.sh that kills the app started by run.sh.'
      onRun(projectName.trim(), enriched)
    }
  }

  return (
    <div className="max-w-3xl mx-auto">
      <h2 className="text-2xl font-bold mb-6">Prompt Input</h2>
      <div className="bg-slate-800 rounded-lg p-4 mb-6">
        <h3 className="text-sm font-medium text-slate-400 mb-2">Configuration Summary</h3>
        <div className="grid grid-cols-3 gap-4 text-sm">
          <div>
            <span className="text-slate-500">Model:</span>{' '}
            <span className="text-white">{modelOption?.label || config.model}</span>
          </div>
          <div>
            <span className="text-slate-500">Workers:</span>{' '}
            <span className="text-white">{config.workerCount}</span>
          </div>
          <div>
            <span className="text-slate-500">Testers:</span>{' '}
            <span className="text-white">{config.testerCount}</span>
          </div>
        </div>
      </div>
      <form onSubmit={handleSubmit} className="space-y-4">
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
            className="w-full px-4 py-3 bg-slate-800 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:border-blue-500 disabled:opacity-50"
          />
        </div>
        <div>
          <label className="block text-sm font-medium text-slate-300 mb-2">
            Prompt
          </label>
          <textarea
            value={prompt}
            onChange={(e) => setPrompt(e.target.value)}
            placeholder="Describe the task for the agents to complete..."
            disabled={isRunning}
            rows={8}
            className="w-full px-4 py-3 bg-slate-800 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:border-blue-500 resize-none disabled:opacity-50"
          />
        </div>
        <div className="flex gap-4">
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
            className="flex-1 px-6 py-3 bg-blue-600 hover:bg-blue-700 disabled:bg-slate-600 disabled:cursor-not-allowed rounded-lg font-semibold transition-colors"
          >
            {isRunning ? 'Starting...' : 'Run'}
          </button>
        </div>
      </form>
    </div>
  )
}

export default PromptPanel
