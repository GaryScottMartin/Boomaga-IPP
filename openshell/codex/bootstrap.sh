#!/usr/bin/env bash
set -euo pipefail

export CODEX_HOME=/sandbox/.codex
mkdir -p "$CODEX_HOME"
chmod 700 "$CODEX_HOME"

# Force predictable file-based credential storage.
CONFIG_FILE="$CODEX_HOME/config.toml"

if [[ ! -f "$CONFIG_FILE" ]]; then
  cat > "$CONFIG_FILE" <<'EOF'
cli_auth_credentials_store = "file"
forced_login_method = "chatgpt"
EOF
  chmod 600 "$CONFIG_FILE"
fi

# Install Codex only when absent.
if ! command -v codex >/dev/null 2>&1; then
  echo "Installing Codex CLI..."

  if command -v npm >/dev/null 2>&1; then
    npm install -g @openai/codex
  else
    curl -fsSL https://chatgpt.com/codex/install.sh | sh
    export PATH="$HOME/.local/bin:$PATH"
  fi
fi

echo "Codex version: $(codex --version)"

# Authenticate only when necessary.
if ! codex login status >/dev/null 2>&1; then
  echo
  echo "Codex authentication is required."
  echo "Complete the device-code flow in your desktop browser."
  echo
  codex login --device-auth
fi

echo
codex login status
echo "Codex is ready."
