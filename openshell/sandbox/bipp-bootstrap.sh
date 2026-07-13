#!/bin/bash
#
# Boomaga-IPP sandbox first-run bootstrap.
#
# Idempotently clones the repo into $BIPP_DIR on first sandbox start, then hands
# off to the agent command. Safe to re-run: on a persistent/restarted sandbox
# where the repo is already present, it is left untouched (unless BIPP_UPDATE=1).
#
# Invoked by ABSOLUTE PATH as the OpenShell entry command — the supervisor runs
# the `--` command over ssh with a near-empty PATH (omits /usr/local/bin AND
# /usr/bin), so neither a bare `bipp-bootstrap` nor `#!/usr/bin/env bash` would
# resolve. Hence `#!/bin/bash` (absolute) and the PATH export below.
#   openshell sandbox create --from ./openshell/sandbox/ ... \
#     -- /usr/local/bin/bipp-bootstrap        # defaults to launching `claude`
#     -- /usr/local/bin/bipp-bootstrap bash   # or pass an explicit command
#
# Overridable via --env at create time:
#   BIPP_REPO_URL  default: https://github.com/GaryScottMartin/Boomaga-IPP.git
#   BIPP_DIR       default: /sandbox/BIPP
#   BIPP_UPDATE    default: 0  (set 1 to `git pull --ff-only` when already cloned)

set -euo pipefail

# The entry command inherits a near-empty PATH from the ssh session, so set a
# sane one before calling git / claude / the passed command.
export PATH="/usr/local/bin:/usr/bin:/bin${PATH:+:$PATH}"

REPO_URL="${BIPP_REPO_URL:-https://github.com/GaryScottMartin/Boomaga-IPP.git}"
DIR="${BIPP_DIR:-/sandbox/BIPP}"

# The enforced sandbox policy (BIPP-project-policy.yaml) matches the repo's git
# paths WITH the ".git" suffix — a bare URL 403s — so keep .git on REPO_URL.
if [ ! -e "$DIR/.git" ]; then
  if [ -e "$DIR" ] && [ -n "$(ls -A "$DIR" 2>/dev/null)" ]; then
    echo "[bipp-bootstrap] ERROR: $DIR exists and is non-empty but is not a git repo." >&2
    echo "[bipp-bootstrap] Remove it or point BIPP_DIR elsewhere, then recreate." >&2
    exit 1
  fi
  echo "[bipp-bootstrap] cloning $REPO_URL -> $DIR"
  git clone "$REPO_URL" "$DIR"
elif [ "${BIPP_UPDATE:-0}" = "1" ]; then
  echo "[bipp-bootstrap] updating $DIR (git pull --ff-only)"
  git -C "$DIR" pull --ff-only || echo "[bipp-bootstrap] pull skipped (non-ff or offline)"
else
  echo "[bipp-bootstrap] $DIR already present; leaving as-is"
fi

# cd into the repo so the agent's project root is the repo — this is also what
# makes project-level .claude/commands (e.g. /handoff) discoverable.
cd "$DIR"

# Hand off to the agent. exec so it becomes the process the supervisor tracks.
if [ "$#" -gt 0 ]; then
  exec "$@"
else
  exec claude
fi
