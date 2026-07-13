# syntax=docker/dockerfile:1
#
# Custom Boomaga-IPP sandbox image for OpenShell.
#
# Build + use in one step (the image is built into the local Docker daemon
# before the sandbox is created):
#
#   openshell sandbox create \
#     --name BIPP \
#     --from ./openshell/sandbox/ \
#     --policy ./BIPP-project-policy.yaml \
#     --provider claude-code --provider github-BIPP \
#     -- bipp-bootstrap
#
# The bootstrap script clones the repo into /sandbox/BIPP on first start and
# then execs the agent (default: `claude`). See ./README.md.

# FROM the OpenShell community base sandbox — the same default the gateway uses
# (gateway.toml: drivers.docker.default_image). Override if your gateway pins a
# different base:  --from is a build, so pass --build-arg via a wrapper if needed.
ARG BASE_IMAGE=ghcr.io/nvidia/openshell-community/sandboxes/base:latest
FROM ${BASE_IMAGE}

# git is required for the first-run clone. The base image already ships it, but
# install defensively so the image still works if a future base drops it.
# (Build runs in the host Docker daemon, NOT under the sandbox network policy,
# so apt has unrestricted network here.)
USER root
RUN if ! command -v git >/dev/null 2>&1; then \
      if command -v apt-get >/dev/null 2>&1; then \
        apt-get update && apt-get install -y --no-install-recommends git && \
        rm -rf /var/lib/apt/lists/*; \
      else \
        echo "git missing and no apt-get available; add git to the base image" >&2; \
        exit 1; \
      fi; \
    fi

# Install the idempotent first-run bootstrap on PATH.
COPY bipp-bootstrap.sh /usr/local/bin/bipp-bootstrap
RUN chmod +x /usr/local/bin/bipp-bootstrap

# Defaults; override at create time with --env KEY=VALUE.
ENV BIPP_REPO_URL="https://github.com/GaryScottMartin/Boomaga-IPP.git" \
    BIPP_DIR="/sandbox/BIPP"

# NOTE: we deliberately do NOT set ENTRYPOINT/CMD to wrap the agent. OpenShell's
# supervisor owns the container's main process and launches the command you pass
# after `--`; an image ENTRYPOINT would not reliably wrap it. Pass the bootstrap
# as the entry command instead (see the header / README).
