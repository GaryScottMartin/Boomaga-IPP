# OpenShell Codex environment notes

- Target OpenShell version: 0.0.86.
- The gateway runs locally.
- Sandboxes remain listed after a gateway restart but become stuck in
  provisioning and must be deleted and recreated.
- The NVIDIA base image included Codex 0.117.0.
- Codex needs to be updated while building the custom image.
- The runtime PATH observed through `openshell sandbox exec` was:

      /sandbox/.venv/bin:/usr/local/bin:/usr/bin:/bin

- Rust was initially installed under `/opt`, but the runtime sandbox user
  could not traverse `/opt`.
- Do not rely on `.bashrc` to expose Rust through OpenShell exec.
- Rust should be installed somewhere accessible through the runtime PATH,
  probably under `/usr/local`.
