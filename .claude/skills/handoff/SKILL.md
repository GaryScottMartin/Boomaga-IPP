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
- **Auto-memory directory**: `/home/gary/.claude/projects/-home-gary-Applications-Boomaga-IPP-Project-Claude-boomaga-ipp/memory/`
- **Session context**: Used for next session startup

## Current Project Context

This is the Boomaga-IPP project - a C++ project that manages IPP (Internet Printing Protocol) printer jobs.

## Examples

```
/handoff
```

This will create a summary of your current work and allow the next session to quickly get up to speed.
