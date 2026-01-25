# Multi-Agent Verse - Requirements and Plan

## Overview

Multi-Agent Verse is a multi-agent orchestrator system with a Rust backend and TypeScript/Bun/TanStack/React 19 frontend. The system uses CLI agents (not APIs) to execute tasks in parallel, following the same pattern as local-agent-orama.

## Agent Types

1. **task-splitter** - Single instance. Receives the user prompt and breaks it into N parallelizable tasks.
2. **coordinator** - Single instance. Orchestrates the execution flow between workers and testers.
3. **worker** - Multiple instances (user-defined count). Executes the actual coding tasks.
4. **tester** - Multiple instances (user-defined count). Tests and validates the work done by workers.

## Model Options

| Agent CLI | Model Name |
|-----------|------------|
| claude-code | opus-4-5 |
| codex | gpt-5.2 |
| copilot | claude-sonnet-4 |
| gemini | gemini-3 (default) |
| llr3 | llama3 |

## Frontend Requirements

### Tab 1 - Configuration

- Display a TREE visualization showing the agent hierarchy:
  ```
  task-splitter (1)
      └── coordinator (1)
              ├── worker (N)
              └── tester (N)
  ```
- User selects the agent/model from dropdown (claude-code/opus-4-5, codex/gpt-5.2, copilot/claude-sonnet-4, gemini/gemini-3, llr3/llama3)
- User inputs the number of worker instances (slider or number input)
- User inputs the number of tester instances (slider or number input)
- "Next" button to proceed to Tab 2

### Tab 2 - Prompt Input

- Text input for **project name** (required)
- Large text area for the user to enter the main prompt
- Display the selected configuration summary (model, worker count, tester count)
- The user's prompt is automatically enriched before sending to the backend. The enrichment appends: "make sure the app works and have a run.sh that run the app, always run on the port 5678 and /index.html. also create a stop.sh that kills the app started by run.sh."
- "Run" button that:
  1. Sends the enriched prompt and project name to backend
  2. task-splitter agent processes the prompt and breaks it into N tasks
  3. Tasks must be splittable and able to run in parallel
  4. Automatically transitions to Tab 3

### Tab 3 - Execution Monitor (Split View)

Screen is split into two panels:

**Left Panel - Agent Tree (50%)**
- Display a TREE visualization with all agents:
  ```
  task-splitter [status]
      └── coordinator [status]
              ├── worker-1 [status]
              ├── worker-2 [status]
              ├── ...
              ├── tester-1 [status]
              ├── tester-2 [status]
              └── ...
  ```
- Each agent node is clickable
- When clicking an agent, show a modal/drawer with:
  - Agent name and model
  - Current status (pending, running, done, error)
  - Log output (stdout/stderr)
  - Error message if failed
- Real-time status updates via polling

**Right Panel - Task List (50%)**
- Display tasks as a TODO list with checkboxes
- Each task shows:
  - Task ID
  - Task description
  - Assigned worker (if any)
  - Assigned tester (if any)
  - Status indicator (pending, in-progress, testing, done, failed)
- Tasks update in real-time as workers/testers complete them
- Visual distinction between:
  - Pending tasks (gray)
  - In-progress tasks (blue)
  - Testing tasks (yellow)
  - Completed tasks (green, with checkmark)
  - Failed tasks (red, with X)

### Tab 4 - Preview

Screen is split into two panels (50/50):

**Left Panel - Project List (50%)**
- Lists all projects found in the `solutions/` folder
- Each project name is clickable
- Shows project name as a list item
- Highlights the currently selected project

**Right Panel - Solution Preview (50%)**
- Displays an iframe that renders the selected solution
- When a project is clicked on the left:
  1. Backend starts the project's `run.sh` (inside `solutions/{project_name}/code/`)
  2. The iframe loads `http://localhost:5678/index.html`
- If no project is selected, shows a placeholder message
- Includes a stop button to stop the currently running preview

### API Endpoints for Preview

```
GET /api/projects
  Response: { projects: string[] }

POST /api/preview/start/{project_name}
  Response: { ok: bool, url: string }

POST /api/preview/stop
  Response: { ok: bool }
```

## Observability

### Frontend Observability
- Real-time status indicators on all agents (color-coded)
- Progress bar showing overall completion percentage
- Event log panel showing timeline of events:
  - "[timestamp] task-splitter started"
  - "[timestamp] task-splitter completed - 5 tasks created"
  - "[timestamp] worker-1 started task #1"
  - "[timestamp] worker-1 completed task #1"
  - "[timestamp] tester-1 testing task #1"
  - "[timestamp] tester-1 approved task #1"
- Toast notifications for important events (task completed, error occurred)
- Agent execution duration displayed on each node

### Backend Observability
- All events logged to `solutions/{project_name}/events.log`
- Structured JSON logs with timestamps
- Log levels: INFO, WARN, ERROR
- Each agent writes logs to `solutions/{project_name}/{agent_id}/logs.txt`
- Summary file generated at end: `solutions/{project_name}/summary.json`

## Solutions Directory

All generated code and artifacts are saved in the `solutions/` folder:

```
solutions/
└── {project_name}/
    ├── events.log              # Timeline of all events
    ├── summary.json            # Final summary with stats
    ├── tasks.json              # Task definitions from task-splitter
    ├── code/                   # All generated code goes here (shared by all workers)
    │   └── ...                 # Project files generated by agents
    ├── task-splitter/
    │   ├── prompt.md
    │   └── logs.txt
    ├── coordinator/
    │   └── logs.txt
    ├── worker-1/
    │   ├── prompt.md
    │   └── logs.txt
    ├── worker-2/
    │   └── ...
    ├── tester-1/
    │   ├── prompt.md
    │   └── logs.txt
    └── tester-2/
        └── ...
```

## Backend Requirements (Rust)

### Technology Stack

- Actix-web for HTTP server
- Tokio for async runtime
- Serde for serialization
- UUID for session management

### API Endpoints

```
POST /api/session
  Request: { model: string, worker_count: number, tester_count: number }
  Response: { session_id: string }

POST /api/run/{session_id}
  Request: { prompt: string, project_name: string }
  Response: { ok: bool }

GET /api/status/{session_id}
  Response: {
    task_splitter: AgentInfo,
    coordinator: AgentInfo,
    workers: AgentInfo[],
    testers: AgentInfo[],
    progress: number,
    elapsed_time: number
  }

GET /api/logs/{session_id}/{agent_id}
  Response: { logs: string, status: string, error: string | null, duration: number }

GET /api/tasks/{session_id}
  Response: { tasks: Task[] }

GET /api/events/{session_id}
  Response: { events: Event[] }

GET /api/projects
  Response: { projects: string[] }

POST /api/preview/start/{project_name}
  Response: { ok: bool, url: string }

POST /api/preview/stop
  Response: { ok: bool }
```

### Agent Execution with Model Parameter

Each agent runs as a CLI subprocess with the model passed as argument:

```rust
// claude-code with model parameter
Command::new("claude")
    .args(["-p", prompt, "--model", model, "--dangerously-skip-permissions"])
    .current_dir(&worktree)
    .output()

// codex with model parameter
Command::new("codex")
    .args(["exec", "--full-auto", "--model", model, prompt])
    .current_dir(&worktree)
    .output()

// copilot with model parameter (e.g. claude-sonnet-4)
Command::new("copilot")
    .args(["--allow-all", "--model", model, "-p", prompt])
    .current_dir(&worktree)
    .output()

// gemini uses default model (gemini 3)
Command::new("gemini")
    .args(["-y", prompt])
    .current_dir(&worktree)
    .output()

// llr3 uses local llama3 model
Command::new("llr3")
    .args(["-p", prompt])
    .current_dir(&worktree)
    .output()
```

### Workflow

1. User creates session with model and instance counts
2. User submits prompt with project name
3. Backend creates `solutions/{project_name}/` directory
4. Backend spawns task-splitter agent with the prompt
5. task-splitter outputs a JSON with N tasks, saved to `solutions/{project_name}/tasks.json`
6. coordinator receives tasks and assigns to workers
7. Workers execute tasks in parallel (each in own directory under solutions)
8. Testers validate completed work
9. All status updates logged and available via polling
10. Final summary generated at `solutions/{project_name}/summary.json`

### Data Models

```rust
enum AgentStatus {
    Pending,
    Running,
    Done,
    Error,
    Timeout,
}

enum AgentRole {
    TaskSplitter,
    Coordinator,
    Worker,
    Tester,
}

struct AgentInfo {
    id: String,
    role: AgentRole,
    model: String,
    status: AgentStatus,
    worktree: PathBuf,
    started_at: Option<DateTime>,
    finished_at: Option<DateTime>,
}

enum TaskStatus {
    Pending,
    InProgress,
    Testing,
    Done,
    Failed,
}

struct Task {
    id: String,
    description: String,
    assigned_worker: Option<String>,
    assigned_tester: Option<String>,
    status: TaskStatus,
    created_at: DateTime,
    updated_at: DateTime,
}

struct Event {
    timestamp: DateTime,
    level: String,
    agent_id: Option<String>,
    message: String,
}

struct Session {
    id: String,
    project_name: String,
    model: String,
    task_splitter: AgentInfo,
    coordinator: AgentInfo,
    workers: Vec<AgentInfo>,
    testers: Vec<AgentInfo>,
    tasks: Vec<Task>,
    events: Vec<Event>,
    created_at: DateTime,
}
```

### Solutions Directory Management

- All agent work happens inside `solutions/{project_name}/`
- No git worktrees needed - each agent has its own subdirectory for logs
- All generated code goes to `solutions/{project_name}/code/` (shared by all workers and testers)
- Agent logs go to `solutions/{project_name}/{agent-id}/logs.txt`

## Project Structure

```
multi-agent-verse/
├── backend/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── agents/
│       │   ├── mod.rs
│       │   ├── claude.rs
│       │   ├── codex.rs
│       │   ├── copilot.rs
│       │   ├── gemini.rs
│       │   └── llr3.rs
│       ├── models/
│       │   └── mod.rs
│       ├── routes/
│       │   └── mod.rs
│       ├── solutions/
│       │   └── mod.rs
│       └── orchestrator/
│           └── mod.rs
├── frontend/
│   ├── package.json
│   ├── tsconfig.json
│   ├── vite.config.ts
│   └── src/
│       ├── main.tsx
│       ├── App.tsx
│       ├── index.css
│       ├── api/
│       │   └── client.ts
│       ├── components/
│       │   ├── AgentTree.tsx
│       │   ├── AgentNode.tsx
│       │   ├── ConfigPanel.tsx
│       │   ├── PromptPanel.tsx
│       │   ├── MonitorPanel.tsx
│       │   ├── TaskList.tsx
│       │   ├── EventLog.tsx
│       │   ├── ProgressBar.tsx
│       │   ├── LogViewer.tsx
│       │   └── PreviewPanel.tsx
│       └── routes/
│           └── index.tsx
├── solutions/                   # Generated solutions stored here
├── run.sh
├── stop.sh
└── goals.md
```

## Scripts

### run.sh
- Build backend with cargo
- Install frontend dependencies with bun
- Create solutions/ directory if not exists
- Start backend server on port 8080
- Start frontend dev server on port 5173

### stop.sh
- Kill backend and frontend processes
- Clean up PID files

## Key Differences from local-agent-orama

1. **Hierarchical Agent Structure** - Not flat, has task-splitter -> coordinator -> workers/testers
2. **Dynamic Instance Count** - User defines how many workers/testers
3. **Task Splitting** - Prompt is split into parallelizable tasks
4. **Four-Tab UI** - Configuration, Prompt, Monitor, Preview
5. **Tree Visualization** - Shows agent hierarchy with clickable nodes
6. **Orchestration Logic** - Coordinator manages task assignment and flow
7. **Split Monitor View** - Left panel for agents, right panel for task list
8. **Full Observability** - Event log, progress tracking, duration metrics
9. **Model as CLI Parameter** - Model passed to agents via command line
10. **Solutions Directory** - All generated code saved in solutions/{project_name}/
11. **Project Name Required** - User must provide project name in Tab 2
12. **Preview Tab** - Iframe-based preview of generated solutions running on port 5678
13. **Prompt Enrichment** - User prompt automatically enriched with run.sh/port 5678 requirements
