import { useState } from 'react'
import { MODEL_OPTIONS } from '../api/client'
import AgentTree from './AgentTree'

interface ConfigPanelProps {
  onNext: (config: {
    model: string
    cliAgent: string
    workerCount: number
    testerCount: number
  }) => void
}

function ConfigPanel({ onNext }: ConfigPanelProps) {
  const [selectedModel, setSelectedModel] = useState(0)
  const [workerCount, setWorkerCount] = useState(3)
  const [testerCount, setTesterCount] = useState(2)

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    const option = MODEL_OPTIONS[selectedModel]
    onNext({
      model: option.model,
      cliAgent: option.cli_agent,
      workerCount,
      testerCount,
    })
  }

  return (
    <div className="max-w-4xl mx-auto">
      <h2 className="text-2xl font-bold mb-6">Configuration</h2>
      <div className="grid grid-cols-2 gap-8">
        <div className="space-y-6">
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Agent / Model
            </label>
            <select
              value={selectedModel}
              onChange={(e) => setSelectedModel(Number(e.target.value))}
              className="w-full px-4 py-3 bg-slate-800 border border-slate-600 rounded-lg text-white focus:outline-none focus:border-blue-500"
            >
              {MODEL_OPTIONS.map((option, index) => (
                <option key={option.cli_agent} value={index}>
                  {option.label}
                </option>
              ))}
            </select>
          </div>
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Number of Workers: {workerCount}
            </label>
            <input
              type="range"
              min="1"
              max="10"
              value={workerCount}
              onChange={(e) => setWorkerCount(Number(e.target.value))}
              className="w-full h-2 bg-slate-700 rounded-lg appearance-none cursor-pointer"
            />
            <div className="flex justify-between text-xs text-slate-500 mt-1">
              <span>1</span>
              <span>10</span>
            </div>
          </div>
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Number of Testers: {testerCount}
            </label>
            <input
              type="range"
              min="1"
              max="10"
              value={testerCount}
              onChange={(e) => setTesterCount(Number(e.target.value))}
              className="w-full h-2 bg-slate-700 rounded-lg appearance-none cursor-pointer"
            />
            <div className="flex justify-between text-xs text-slate-500 mt-1">
              <span>1</span>
              <span>10</span>
            </div>
          </div>
          <button
            onClick={handleSubmit}
            className="w-full px-6 py-3 bg-blue-600 hover:bg-blue-700 rounded-lg font-semibold transition-colors"
          >
            Next
          </button>
        </div>
        <div>
          <AgentTree workerCount={workerCount} testerCount={testerCount} />
        </div>
      </div>
    </div>
  )
}

export default ConfigPanel
