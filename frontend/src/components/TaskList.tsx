import { Task, TaskStatus } from '../api/client'

interface TaskListProps {
  tasks: Task[]
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

function TaskList({ tasks }: TaskListProps) {
  if (tasks.length === 0) {
    return (
      <div className="h-full flex items-center justify-center text-slate-500">
        No tasks yet. Waiting for task-splitter to generate tasks...
      </div>
    )
  }

  return (
    <div className="space-y-2">
      {tasks.map((task) => {
        const styles = getStatusStyles(task.status)
        return (
          <div
            key={task.id}
            className={`p-3 rounded-lg border border-slate-700 ${styles.bg}`}
          >
            <div className="flex items-start gap-3">
              <span className={`font-mono text-lg ${styles.text}`}>
                {styles.icon}
              </span>
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <span className="font-semibold text-white">
                    Task #{task.id}
                  </span>
                  <span className={`text-xs px-2 py-0.5 rounded ${styles.bg} ${styles.text} uppercase`}>
                    {task.status}
                  </span>
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
      })}
    </div>
  )
}

export default TaskList
