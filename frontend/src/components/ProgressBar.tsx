interface ProgressBarProps {
  progress: number
  elapsedTime: number
}

function formatTime(seconds: number): string {
  const mins = Math.floor(seconds / 60)
  const secs = seconds % 60
  return `${mins}:${secs.toString().padStart(2, '0')}`
}

function ProgressBar({ progress, elapsedTime }: ProgressBarProps) {
  return (
    <div className="bg-slate-800 rounded-lg p-4">
      <div className="flex justify-between text-sm text-slate-400 mb-2">
        <span>Progress: {progress.toFixed(1)}%</span>
        <span>Elapsed: {formatTime(elapsedTime)}</span>
      </div>
      <div className="h-3 bg-slate-700 rounded-full overflow-hidden">
        <div
          className="h-full bg-blue-500 transition-all duration-300"
          style={{ width: `${progress}%` }}
        />
      </div>
    </div>
  )
}

export default ProgressBar
