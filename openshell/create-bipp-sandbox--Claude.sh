#!/bin/bash
#
# Host-side helper — run this on the OpenShell host (e.g. Denali) to create a
# BIPP sandbox that auto-clones the repo into /sandbox/BIPP and launches claude.
#
# WHY a script (not a baked image, not a pasted one-liner):
#   * OpenShell does not serve a `--from` image's /usr/local/bin at runtime — a
#     script COPY'd into the image is absent in the running sandbox — so
#     provisioning must go through the `--` entry command, not a baked file.
#   * A long `--` command line-wraps in the terminal, and the inserted newlines
#     split tokens (e.g. `git clone` from its URL). Keeping it in a file avoids
#     that: the entry command is passed as one clean argv element.
#
# USAGE (run from the repo root so ./openshell/BIPP-project-policy.yaml resolves):
#   ./openshell/create-bipp-sandbox.sh [sandbox-name]   # create + launch claude
#   BIPP_VERIFY=1 ./openshell/create-bipp-sandbox.sh    # non-interactive smoke test (--no-keep)

set -euo pipefail

DIR="/sandbox/BIPP"
# Keep the ".git" suffix — the enforced policy 403s a bare URL.
REPO_URL="https://github.com/GaryScottMartin/Boomaga-IPP.git"
CLONE="[ -e $DIR/.git ] || git clone $REPO_URL $DIR"

if [ -n "${BIPP_VERIFY:-}" ]; then
  NAME="BIPP-verify"
  EXTRA=(--no-keep)
  # Print the three pass-criteria markers, then run the SessionStart hook exactly
  # as a fresh-clone claude launch would (no interactive agent).
  ENTRY="$CLONE; cd $DIR; echo \"PWD=\$(pwd)\"; test -d .git && echo GIT_OK; ls .claude/commands/; echo '--- SessionStart hook ---'; .claude/hooks/session-start.sh"
else
  NAME="${1:-BIPP}"
  EXTRA=()
  ENTRY="$CLONE; cd $DIR; exec claude"
fi

exec openshell sandbox create \
  --name "$NAME" \
  --policy ./openshell/BIPP-project-policy--Claude.yaml \
  --provider claude-code --provider github-BIPP \
  "${EXTRA[@]}" \
  -- bash -lc "$ENTRY"
