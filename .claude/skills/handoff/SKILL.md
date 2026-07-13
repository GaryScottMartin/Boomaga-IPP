# Handoff Skill

This skill helps save and restore session context when ending a Claude Code session.

## Usage

When you are in the middle of work and need to end a session, use this skill to:

1. **Create a handoff summary** of current work
2. **Save context** for the next session to continue efficiently
3. **Clear unnecessary state** to start fresh

## How It Works

The handoff skill generates a comprehensive summary of:
- Current tasks in progress
- Recent work completed
- Context about the codebase and project
- Important decisions made
- Next steps to continue

## Handoff Summary Location

Handoff summaries are saved in:
- **In-repo handoff**: `docs/HANDOFF.md` (version-controlled, shared across clones)
- **Auto-memory directory**: your Claude Code project memory dir
  (`~/.claude/projects/<project-slug>/memory/`) — this path is per-machine/per-user
- **Session context**: Used for next session startup

## Current Project Context

This is the Boomaga-IPP project - a Rust rewrite of the Boomaga virtual printer with IPP Everywhere and native Wayland support.

## Examples

```
/handoff
```

This will create a summary of your current work and allow the next session to quickly get up to speed.
