#!/usr/bin/env bash

  set -euo pipefail

  readonly GATEWAY_STARTER="/home/gary/bin/openshell-gateway-up.sh"
  readonly PROJECT_DIR="/home/gary/Applications/Boomaga-IPP/Project/Claude/boomaga-ipp"
  readonly SANDBOX_STARTER="$PROJECT_DIR/openshell/codex/create-bipp-sandbox--Codex.sh"

  readonly STARTUP_TIMEOUT=120
  readonly POLL_INTERVAL=2

  gateway_is_ready() {
      timeout 5s openshell sandbox list >/dev/null 2>&1
  }

  die() {
      printf 'Error: %s\n' "$*" >&2
      exit 1
  }

  command -v openshell >/dev/null 2>&1 ||
      die "'openshell' is not available in PATH."

  command -v konsole >/dev/null 2>&1 ||
      die "'konsole' is not available in PATH."

  command -v timeout >/dev/null 2>&1 ||
      die "'timeout' is not available in PATH."

  [[ -x "$GATEWAY_STARTER" ]] ||
      die "gateway starter is missing or not executable: $GATEWAY_STARTER"

  [[ -d "$PROJECT_DIR" ]] ||
      die "project directory does not exist: $PROJECT_DIR"

  [[ -x "$SANDBOX_STARTER" ]] ||
      die "sandbox starter is missing or not executable: $SANDBOX_STARTER"

  if gateway_is_ready; then
      printf 'OpenShell gateway is already available.\n'
  else
      printf 'OpenShell gateway is not available; starting it now.\n'

      "$GATEWAY_STARTER"

      printf 'Waiting for the OpenShell gateway'

      elapsed=0
      until gateway_is_ready; do
          if (( elapsed >= STARTUP_TIMEOUT )); then
              printf '\n' >&2
              die "gateway did not become available within ${STARTUP_TIMEOUT}
              seconds."
          fi

          printf '.'
          sleep "$POLL_INTERVAL"
          (( elapsed += POLL_INTERVAL ))
      done

      printf ' ready.\n'
  fi

  printf 'Opening the BIPP Codex sandbox in a new Konsole tab.\n'

  konsole \
      --new-tab \
      --workdir "$PROJECT_DIR" \
      -e bash -lc '
          sandbox_starter=$1

          old_userns_limit=$(sysctl -n user.max_user_namespaces)
          old_clone_setting=$(
              sysctl -n kernel.unprivileged_userns_clone 2>/dev/null || true
          )

          restore_userns() {
              status=$?

              # Prevent recursion if restoration or authentication fails.
              trap - EXIT INT TERM
              set +e

              printf "\nRestoring host user-namespace settings.\n"

              # Require fresh authentication instead of relying on the startup
              # sudo timestamp.
              sudo -k
              if ! sudo -v; then
                  printf \
                      "ERROR: sudo authentication failed; host sysctls were not
                      restored.\n" \
                      >&2

                  if [[ -n "$old_clone_setting" ]]; then
                      printf \
                          "Run: sudo sysctl -w
                          kernel.unprivileged_userns_clone=%s\n" \
                          "$old_clone_setting" >&2
                  fi

                  printf \
                      "Run: sudo sysctl -w user.max_user_namespaces=%s\n" \
                      "$old_userns_limit" >&2

                  read -r -p "Press Enter to close this terminal..."
                  exit 1
              fi

              restore_failed=0

              # Restore the security-sensitive setting first.
              if [[ -n "$old_clone_setting" ]]; then
                  sudo -n sysctl -q -w \
                      "kernel.unprivileged_userns_clone=$old_clone_setting" ||
                      restore_failed=1
              fi

              sudo -n sysctl -q -w \
                  "user.max_user_namespaces=$old_userns_limit" ||
                  restore_failed=1

              if (( restore_failed )); then
                  printf \
                      "ERROR: one or more host sysctls were not restored.\n" \
                      >&2
                  read -r -p "Press Enter to close this terminal..."
                  exit 1
              fi

              printf "Host user-namespace settings restored.\n"
              exit "$status"
          }

          trap restore_userns EXIT
          trap "exit 130" INT
          trap "exit 143" TERM

          sudo sysctl -q -w user.max_user_namespaces=49152

          if [[ -n "$old_clone_setting" ]]; then
              sudo sysctl -q -w kernel.unprivileged_userns_clone=1
          fi

          # Temporarily disable errexit so the sandbox status can be captured.
          set +e
          "$sandbox_starter"
          status=$?
          set -e

          printf "\nSandbox process exited with status %d.\n" "$status"
          exit "$status"
      ' bash "$SANDBOX_STARTER"
