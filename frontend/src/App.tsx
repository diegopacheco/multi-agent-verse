import { useState, useEffect, useCallback } from 'react'
import ConfigPanel from './components/ConfigPanel'
import PromptPanel from './components/PromptPanel'
import MonitorPanel from './components/MonitorPanel'
import PreviewPanel from './components/PreviewPanel'
import { createSession, runSession } from './api/client'
import logoImg from './assets/logo.png'

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
  const [showHelp, setShowHelp] = useState(false)

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

  const navigateToTab = useCallback((tab: Tab) => {
    if (tab === 'prompt' && !config) return
    if (tab === 'monitor' && !sessionId) return
    setActiveTab(tab)
  }, [config, sessionId])

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement
      if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') return
      
      switch (e.key) {
        case '1':
        case 'a':
          navigateToTab('config')
          break
        case '2':
        case 'p':
          navigateToTab('prompt')
          break
        case '3':
        case 't':
          navigateToTab('monitor')
          break
        case '4':
        case 'o':
          navigateToTab('preview')
          break
        case '?':
          setShowHelp(prev => !prev)
          break
        case 'Escape':
          setShowHelp(false)
          break
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [navigateToTab])

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
    navigateToTab(tab)
  }

  return (
    <div className="h-screen flex flex-col overflow-hidden">
      {showHelp && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/80">
          <div className="bg-slate-800 rounded-lg p-6 max-w-md">
            <h2 className="text-xl font-bold mb-4">Keyboard Shortcuts</h2>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between"><span>1 or a</span><span className="text-slate-400">Configuration tab</span></div>
              <div className="flex justify-between"><span>2 or p</span><span className="text-slate-400">Prompt tab</span></div>
              <div className="flex justify-between"><span>3 or t</span><span className="text-slate-400">Monitor (Tasks) tab</span></div>
              <div className="flex justify-between"><span>4 or o</span><span className="text-slate-400">Preview tab</span></div>
              <div className="flex justify-between"><span>?</span><span className="text-slate-400">Toggle this help</span></div>
              <div className="flex justify-between"><span>ESC</span><span className="text-slate-400">Close dialogs</span></div>
            </div>
            <button
              onClick={() => setShowHelp(false)}
              className="mt-4 w-full py-2 bg-blue-600 hover:bg-blue-700 rounded"
            >
              Close (ESC)
            </button>
          </div>
        </div>
      )}
      <header className="flex-shrink-0 py-4 px-8 border-b border-slate-800">
        <div className="max-w-6xl mx-auto flex items-center gap-4">
          <img src={logoImg} alt="Logo" className="h-12 w-12 opacity-80" />
          <div>
            <h1 className="text-2xl font-bold">Multi-Agent Verse</h1>
            <p className="text-slate-400 text-xs">
              Multi-agent orchestrator with task splitting and parallel execution
            </p>
          </div>
          <button
            onClick={() => setShowHelp(true)}
            className="ml-auto text-slate-500 hover:text-white text-sm"
            title="Keyboard shortcuts"
          >
            ? Help
          </button>
        </div>
      </header>
      <nav className="flex-shrink-0 border-b border-slate-800">
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
      <main className="flex-1 p-8 overflow-hidden">
        <div className="max-w-6xl mx-auto h-full">
          <div className="h-full" style={{ display: activeTab === 'config' ? 'block' : 'none' }}>
            <ConfigPanel onNext={handleConfigNext} />
          </div>
          <div className="h-full overflow-auto" style={{ display: activeTab === 'prompt' ? 'block' : 'none' }}>
            {config && (
              <PromptPanel
                config={config}
                onRun={handleRun}
                onBack={handlePromptBack}
                isRunning={isRunning}
              />
            )}
          </div>
          <div className="h-full" style={{ display: activeTab === 'monitor' ? 'block' : 'none' }}>
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
          <div className="h-full" style={{ display: activeTab === 'preview' ? 'block' : 'none' }}>
            <PreviewPanel />
          </div>
        </div>
      </main>
    </div>
  )
}

export default App
