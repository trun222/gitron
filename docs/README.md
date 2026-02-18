# Gitron Documentation

This directory contains all project documentation. Start here to understand the project.

## Documents

| Document | Purpose | Read When |
|----------|---------|-----------|
| [VISION.md](./VISION.md) | Why Gitron exists, core principles, technology decisions, target users | Understanding the project's direction and goals |
| [ARCHITECTURE.md](./ARCHITECTURE.md) | System overview, project structure, module details, plugin/agent API shapes, data flow diagrams | Understanding how the system is structured and how parts connect |
| [TECHNICAL_SPEC.md](./TECHNICAL_SPEC.md) | Complete IPC command reference, state management, event system, data types, error handling, concurrency, security | Implementing features or debugging behavior — this is the definitive reference |
| [PLUGIN_SYSTEM.md](./PLUGIN_SYSTEM.md) | Plugin architecture, backend/frontend APIs, extension points, distribution, security, examples | Building or understanding the plugin system (Phase 4) |
| [AGENT_GATEWAY.md](./AGENT_GATEWAY.md) | MCP server, permission model, action queue, event stream, AI workflows, autonomous agents | Building or understanding the agent system (Phase 5) |
| [DEVELOPER_GUIDE.md](./DEVELOPER_GUIDE.md) | Setup, workflow, module boundaries, how to add features/commands/components, conventions, pitfalls | Working on the codebase — read this first before writing code |
| [ROADMAP.md](./ROADMAP.md) | Phased build plan with checkboxes, current status, exit criteria per phase | Understanding what's done, what's next, and overall progress |

## Reading Order

**For new contributors:**
1. VISION.md — understand what we're building
2. DEVELOPER_GUIDE.md — understand how to work on it
3. ARCHITECTURE.md — understand how it's structured
4. TECHNICAL_SPEC.md — reference as needed while coding

**For AI assistants / future sessions:**
1. ROADMAP.md — check current status and what's next
2. TECHNICAL_SPEC.md — understand the IPC contract and data types
3. DEVELOPER_GUIDE.md — understand conventions and boundaries
4. The specific design doc for the area being worked on

**For plugin developers (future):**
1. PLUGIN_SYSTEM.md — full plugin API reference

**For agent integration (future):**
1. AGENT_GATEWAY.md — full agent gateway reference
