import { useState } from 'react'
import ConfigPanel from './components/ConfigPanel'
import PromptPanel from './components/PromptPanel'
import MonitorPanel from './components/MonitorPanel'
import PreviewPanel from './components/PreviewPanel'
import { createSession, runSession } from './api/client'

type Tab = 'config' | 'prompt' | 'monitor' | 'preview'

interface Config {
  model: string
  cliAgent: string
  workerCount: number
  testerCount: number
}

function App() {
  const [activeTab, setActiveTab] = useState<Tab>('config')
  const [config, setConfig] = useState<Config | null>(null)
  const [sessionId, setSessionId] = useState<string | null>(null)
  const [isRunning, setIsRunning] = useState(false)

  const handleConfigNext = (newConfig: Config) => {
    setConfig(newConfig)
    setActiveTab('prompt')
  }

  const handlePromptBack = () => {
    setActiveTab('config')
  }

  const handleRun = async (projectName: string, prompt: string) => {
    if (!config) return
    setIsRunning(true)
    try {
      const sessionRes = await createSession({
        model: config.model,
        cli_agent: config.cliAgent,
        worker_count: config.workerCount,
        tester_count: config.testerCount,
      })
      setSessionId(sessionRes.session_id)
      await runSession(sessionRes.session_id, {
        prompt,
        project_name: projectName,
      })
      setActiveTab('monitor')
    } catch (error) {
      console.error('Failed to start:', error)
    } finally {
      setIsRunning(false)
    }
  }

  const getTabClass = (tab: Tab) => {
    const base = 'px-6 py-3 font-medium transition-colors'
    if (tab === activeTab) {
      return `${base} text-blue-400 border-b-2 border-blue-400 bg-slate-800`
    }
    const isDisabled =
      (tab === 'prompt' && !config) ||
      (tab === 'monitor' && !sessionId)
    if (isDisabled) {
      return `${base} text-slate-600 cursor-not-allowed`
    }
    return `${base} text-slate-400 hover:text-white hover:bg-slate-800/50`
  }

  const handleTabClick = (tab: Tab) => {
    if (tab === 'prompt' && !config) return
    if (tab === 'monitor' && !sessionId) return
    setActiveTab(tab)
  }

  return (
    <div className="min-h-screen flex flex-col">
      <header className="py-6 px-8 border-b border-slate-800">
        <div className="max-w-6xl mx-auto">
          <h1 className="text-3xl font-bold">Multi-Agent Verse</h1>
          <p className="text-slate-400 text-sm">
            Multi-agent orchestrator with task splitting and parallel execution
          </p>
        </div>
      </header>
      <nav className="border-b border-slate-800">
        <div className="max-w-6xl mx-auto flex">
          <button
            className={getTabClass('config')}
            onClick={() => handleTabClick('config')}
          >
            1. Configuration
          </button>
          <button
            className={getTabClass('prompt')}
            onClick={() => handleTabClick('prompt')}
          >
            2. Prompt
          </button>
          <button
            className={getTabClass('monitor')}
            onClick={() => handleTabClick('monitor')}
          >
            3. Monitor
          </button>
          <button
            className={getTabClass('preview')}
            onClick={() => handleTabClick('preview')}
          >
            4. Preview
          </button>
        </div>
      </nav>
      <main className="flex-1 p-8">
        <div className="max-w-6xl mx-auto h-full">
          <div style={{ display: activeTab === 'config' ? 'block' : 'none' }}>
            <ConfigPanel onNext={handleConfigNext} />
          </div>
          <div style={{ display: activeTab === 'prompt' ? 'block' : 'none' }}>
            {config && (
              <PromptPanel
                config={config}
                onRun={handleRun}
                onBack={handlePromptBack}
                isRunning={isRunning}
              />
            )}
          </div>
          <div style={{ display: activeTab === 'monitor' ? 'block' : 'none' }}>
            {sessionId && config && (
              <MonitorPanel
                sessionId={sessionId}
                config={{
                  workerCount: config.workerCount,
                  testerCount: config.testerCount,
                }}
              />
            )}
          </div>
          <div style={{ display: activeTab === 'preview' ? 'block' : 'none' }}>
            <PreviewPanel />
          </div>
        </div>
      </main>
    </div>
  )
}

export default App
