# Boomaga-IPP OpenShell sandbox image

A custom OpenShell sandbox image that **auto-clones this repo into `/sandbox/BIPP`
on first start** and then launches the agent. It removes the need to inline a
clone into every `openshell sandbox create` call.

## Files
- `Dockerfile` — `FROM` the OpenShell community base, adds `git` (defensively)
  and installs `bipp-bootstrap` on `PATH`.
- `bipp-bootstrap.sh` — idempotent first-run clone + agent hand-off.

## Usage

Run from the repo root (so `./openshell/sandbox/` and `./BIPP-project-policy.yaml`
resolve). `--from <dir>` builds the image into your local Docker daemon first,
then creates the sandbox:

```bash
openshell sandbox create \
  --name BIPP \
  --from ./openshell/sandbox/ \
  --policy ./BIPP-project-policy.yaml \
  --provider claude-code --provider github-BIPP \
  -- bipp-bootstrap
```

`-- bipp-bootstrap` with no trailing command defaults to launching `claude`.
Pass an explicit command to override, e.g. `-- bipp-bootstrap bash`.

### Optional: bring your local personal settings

A fresh clone does **not** include `.claude/settings.local.json` (it is
gitignored — your personal permission allowlist), so a new sandbox re-prompts for
Bash permissions. Layer it back in with `--upload` (needs `--no-git-ignore`
because the file is gitignored):

```bash
  --upload ~/Applications/Boomaga-IPP/Project/Claude/boomaga-ipp/.claude/settings.local.json:/sandbox/BIPP/.claude/settings.local.json \
  --no-git-ignore \
```

## Environment overrides (`--env KEY=VALUE`)
| Var | Default | Meaning |
|-----|---------|---------|
| `BIPP_REPO_URL` | `https://github.com/GaryScottMartin/Boomaga-IPP.git` | Repo to clone. **Keep the `.git` suffix** — the enforced policy 403s a bare URL. |
| `BIPP_DIR` | `/sandbox/BIPP` | Clone destination. |
| `BIPP_UPDATE` | `0` | `1` → `git pull --ff-only` when the repo is already present. |

## Troubleshooting
- **`bipp-bootstrap: command not found` / exit 127.** OpenShell runs the `--`
  entry command over ssh with a lean PATH that omits `/usr/local/bin`. The image
  therefore symlinks the bootstrap into `/usr/bin` (always on PATH). If you ever
  hit this, invoke by absolute path as a guaranteed fallback:
  `-- /usr/local/bin/bipp-bootstrap …`.

## Notes
- **The `cd "$DIR"` in the bootstrap is load-bearing — do not remove it.**
  Claude Code discovers project config (`.claude/commands/` → `/handoff`,
  `.claude/settings.json` → the SessionStart/PreCompact hooks) from its working
  directory at launch. The repo lives in the subdirectory `/sandbox/BIPP`, but
  the sandbox default cwd is `/sandbox`, and there is no `--workdir` flag on
  `sandbox create`. Without the `cd` (e.g. a plain `-- claude`), claude starts in
  `/sandbox`, never sees `/sandbox/BIPP/.claude/`, and `/handoff` is an unknown
  command. The `cd` before `exec claude` is what makes the handoff system work
  from a subdirectory.
- **Fresh clone → empty handoff context on first start (expected).** The context
  files the SessionStart hook loads (`.claude/context.md`, `current-task.md`, …)
  are gitignored, so they don't come down with a clone; the hook prints "No
  handoff context found" until you run `/handoff` once. The committed
  `docs/HANDOFF.md` is the durable cross-sandbox handoff.
- **Idempotent:** on a persistent/restarted sandbox the existing clone is left
  as-is (no re-clone, no clobbering local work) unless `BIPP_UPDATE=1`.
- **Real `.git`:** unlike `--upload`, this yields a working `origin`, so
  `git push` from inside the sandbox works (via `GIT_ASKPASS` + the injected
  `GITHUB_TOKEN`; see `docs/HANDOFF.md` §5).
- **Why the bootstrap is the entry command, not an image `ENTRYPOINT`:**
  OpenShell's supervisor owns the container's main process and runs whatever you
  pass after `--`; an image `ENTRYPOINT` would not reliably wrap the agent.
- **Base image:** pinned via the `BASE_IMAGE` build arg to the gateway default
  (`gateway.toml` → `drivers.docker.default_image`). Update both together if the
  gateway's base changes.
