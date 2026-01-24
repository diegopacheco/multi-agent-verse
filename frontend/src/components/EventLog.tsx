import { Event } from '../api/client'

interface EventLogProps {
  events: Event[]
}

function getLevelColor(level: string): string {
  switch (level.toUpperCase()) {
    case 'ERROR':
      return 'text-red-400'
    case 'WARN':
      return 'text-yellow-400'
    default:
      return 'text-slate-400'
  }
}

function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp)
  return date.toLocaleTimeString()
}

function EventLog({ events }: EventLogProps) {
  if (events.length === 0) {
    return (
      <div className="text-slate-500 text-sm">No events yet...</div>
    )
  }

  return (
    <div className="space-y-1 font-mono text-xs max-h-48 overflow-y-auto">
      {events.map((event, index) => (
        <div key={index} className="flex gap-2">
          <span className="text-slate-600">
            [{formatTimestamp(event.timestamp)}]
          </span>
          <span className={getLevelColor(event.level)}>
            [{event.level}]
          </span>
          <span className="text-slate-400">
            {event.agent_id && (
              <span className="text-blue-400">{event.agent_id}: </span>
            )}
            {event.message}
          </span>
        </div>
      ))}
    </div>
  )
}

export default EventLog
