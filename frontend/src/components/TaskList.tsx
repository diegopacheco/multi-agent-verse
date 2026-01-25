import { Task, TaskStatus, TaskType } from '../api/client'

interface TaskListProps {
  tasks: Task[]
  onTaskClick?: (task: Task) => void
}

function getStatusStyles(status: TaskStatus): { bg: string; text: string; icon: string } {
  switch (status) {
    case 'done':
      return { bg: 'bg-green-900/30', text: 'text-green-400', icon: '[x]' }
    case 'inprogress':
      return { bg: 'bg-blue-900/30', text: 'text-blue-400', icon: '[>]' }
    case 'testing':
      return { bg: 'bg-yellow-900/30', text: 'text-yellow-400', icon: '[?]' }
    case 'failed':
      return { bg: 'bg-red-900/30', text: 'text-red-400', icon: '[!]' }
    default:
      return { bg: 'bg-slate-800', text: 'text-slate-400', icon: '[ ]' }
  }
}

function getTaskTypeStyles(taskType: TaskType): { bg: string; text: string; label: string } {
  switch (taskType) {
    case 'parallel':
      return { bg: 'bg-purple-900/30', text: 'text-purple-400', label: 'PARALLEL' }
    case 'sequential':
      return { bg: 'bg-orange-900/30', text: 'text-orange-400', label: 'SEQUENTIAL' }
    default:
      return { bg: 'bg-slate-800', text: 'text-slate-400', label: 'UNKNOWN' }
  }
}

function TaskCard({ task, onClick }: { task: Task; onClick?: (task: Task) => void }) {
  const statusStyles = getStatusStyles(task.status)
  const typeStyles = getTaskTypeStyles(task.task_type || 'parallel')
  const isClickable = !!onClick && task.assigned_worker
  return (
    <div
      className={`p-3 rounded-lg border border-slate-700 ${statusStyles.bg} ${isClickable ? 'cursor-pointer hover:border-slate-500 transition-colors' : ''}`}
      onClick={() => isClickable && onClick(task)}
    >
      <div className="flex items-start gap-3">
        <span className={`font-mono text-lg ${statusStyles.text}`}>
          {statusStyles.icon}
        </span>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 flex-wrap">
            <span className="font-semibold text-white">
              Task #{task.id}
            </span>
            <span className={`text-xs px-2 py-0.5 rounded ${typeStyles.bg} ${typeStyles.text}`}>
              {typeStyles.label}
            </span>
            {task.task_type === 'sequential' && task.order && (
              <span className="text-xs px-2 py-0.5 rounded bg-slate-700 text-slate-300">
                Order: {task.order}
              </span>
            )}
            <span className={`text-xs px-2 py-0.5 rounded ${statusStyles.bg} ${statusStyles.text} uppercase`}>
              {task.status}
            </span>
            {isClickable && (
              <span className="text-xs text-slate-500">(click for logs)</span>
            )}
          </div>
          <p className="text-sm text-slate-300 mt-1 truncate">
            {task.description}
          </p>
          <div className="flex gap-4 mt-2 text-xs text-slate-500">
            {task.assigned_worker && (
              <span>Worker: {task.assigned_worker}</span>
            )}
            {task.assigned_tester && (
              <span>Tester: {task.assigned_tester}</span>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}

function TaskList({ tasks, onTaskClick }: TaskListProps) {
  if (tasks.length === 0) {
    return (
      <div className="h-full flex items-center justify-center text-slate-500">
        No tasks yet. Waiting for task-splitter to generate tasks...
      </div>
    )
  }

  const parallelTasks = tasks.filter(t => t.task_type === 'parallel' || !t.task_type)
  const sequentialTasks = tasks
    .filter(t => t.task_type === 'sequential')
    .sort((a, b) => (a.order || 0) - (b.order || 0))

  return (
    <div className="space-y-4">
      {parallelTasks.length > 0 && (
        <div>
          <h3 className="text-sm font-semibold text-purple-400 mb-2 flex items-center gap-2">
            <span className="w-2 h-2 rounded-full bg-purple-400"></span>
            Parallel Tasks ({parallelTasks.length})
          </h3>
          <div className="space-y-2">
            {parallelTasks.map((task) => (
              <TaskCard key={task.id} task={task} onClick={onTaskClick} />
            ))}
          </div>
        </div>
      )}
      {sequentialTasks.length > 0 && (
        <div>
          <h3 className="text-sm font-semibold text-orange-400 mb-2 flex items-center gap-2">
            <span className="w-2 h-2 rounded-full bg-orange-400"></span>
            Sequential Tasks ({sequentialTasks.length})
          </h3>
          <div className="space-y-2">
            {sequentialTasks.map((task) => (
              <TaskCard key={task.id} task={task} onClick={onTaskClick} />
            ))}
          </div>
        </div>
      )}
    </div>
  )
}

export default TaskList
