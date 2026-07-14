# OpenShell provisioning for Boomaga-IPP

`create-bipp-sandbox.sh` creates an OpenShell sandbox that **auto-clones this repo
into `/sandbox/BIPP`** and launches `claude` in it — so you don't hand-clone or
inline a long command into every `openshell sandbox create`.

## Usage

Run **on the OpenShell host** (e.g. Denali), from the repo root (so
`./openshell/BIPP-project-policy.yaml` resolves):

```bash
./openshell/create-bipp-sandbox.sh [sandbox-name]   # create + launch claude (default name: BIPP)
BIPP_VERIFY=1 ./openshell/create-bipp-sandbox.sh     # non-interactive smoke test (--no-keep)
```

The verify mode prints `PWD=/sandbox/BIPP`, `GIT_OK`, and `handoff.md`, then runs
the SessionStart hook (so you see the exact context message a fresh-clone launch
would show), then the sandbox self-deletes. (Verified working 2026-07-13.)

Repo URL / clone dir are near the top of the script if you need to change them
(keep the `.git` suffix — see below).

### Optional: bring your local personal settings

A fresh clone does **not** include `.claude/settings.local.json` (it's gitignored —
your personal permission allowlist), so a new sandbox re-prompts for Bash
permissions. Add `--upload` to the `openshell sandbox create` line in the script
(needs `--no-git-ignore` because the file is gitignored):

```bash
  --upload ~/Applications/Boomaga-IPP/Project/Claude/boomaga-ipp/.claude/settings.local.json:/sandbox/BIPP/.claude/settings.local.json \
  --no-git-ignore \
```

## Why a script, and why not a `--from` image

We first tried a custom `--from` sandbox image with the bootstrap baked in. It
**does not work** with this OpenShell setup, confirmed by diagnostics:

- **A file `COPY`'d into the image at `/usr/local/bin` is absent at runtime.**
  `openshell sandbox create --from …` builds the image, but the running sandbox's
  `/usr/local/bin` is not served from it (`ls /usr/local/bin/bipp-bootstrap` →
  *No such file or directory*, while base-image paths like `/sandbox/.venv` are
  present). So a baked-in bootstrap script simply isn't there to run.
- **PATH is fine** at runtime (`/sandbox/.venv/bin:/usr/local/bin:/usr/bin:/bin`),
  so the earlier "command not found" was the missing file, not a PATH problem.
- **A long inline `--` one-liner corrupts on paste** — the terminal wraps it and
  the inserted newlines split tokens (e.g. `git clone` from its URL). A script
  file avoids this: the entry command is passed as one clean argv element.

Provisioning therefore goes through the `--` **entry command**, kept in this script.

## Notes / gotchas

- **Keep the `.git` suffix on the clone URL.** The enforced `BIPP-project-policy.yaml`
  matches `/GaryScottMartin/Boomaga-IPP*/…`; a bare URL 403s at the proxy.
- **The `cd /sandbox/BIPP` is load-bearing.** Claude Code discovers project config
  (`.claude/commands` → `/handoff`, `.claude/settings.json` → hooks) from its launch
  cwd; the repo is a subdir, so the script `cd`s in before `exec claude`. Without it,
  `/handoff` is "unknown command".
- **Real `.git` with a working `origin`** — unlike `--upload` provisioning (which
  yields no `.git`), the clone lets you `git push` from inside the sandbox (via
  `GIT_ASKPASS` + the injected `GITHUB_TOKEN`; see `docs/HANDOFF.md` §5).
- **Fresh clone → empty handoff context on first start** (expected): the context
  files the SessionStart hook loads are gitignored, so `/handoff` reports "no
  context found" until you run it once. `docs/HANDOFF.md` is the durable handoff.
