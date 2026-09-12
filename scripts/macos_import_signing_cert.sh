#!/usr/bin/env bash
# Import APPLE_CERTIFICATE into a job-scoped keychain so post-Tauri codesign
# can find APPLE_SIGNING_IDENTITY. Tauri creates an ephemeral keychain and
# deletes it when tauri-action finishes — resign/notarize need the cert again.
set -euo pipefail

if [[ -z "${APPLE_CERTIFICATE:-}" || -z "${APPLE_CERTIFICATE_PASSWORD:-}" ]]; then
  echo "No APPLE_CERTIFICATE; skipping keychain import (ad-hoc / local keychain assumed)."
  exit 0
fi

KEYCHAIN_PASSWORD="${KEYCHAIN_PASSWORD:-bellanote-ci}"
tmp="${RUNNER_TEMP:-${TMPDIR:-/tmp}}"
KEYCHAIN="$tmp/bellanote-signing.keychain-db"
CERT_PATH="$tmp/bellanote-signing.p12"

cleanup_cert() {
  rm -f "$CERT_PATH"
}
trap cleanup_cert EXIT

echo "$APPLE_CERTIFICATE" | base64 --decode > "$CERT_PATH"
security delete-keychain "$KEYCHAIN" 2>/dev/null || true
security create-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN"
security set-keychain-settings -lut 21600 "$KEYCHAIN"
security unlock-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN"

# Prefer our keychain first; keep existing user keychains after it.
existing="$(security list-keychains -d user | sed -e 's/"//g')"
# shellcheck disable=SC2086
security list-keychains -d user -s "$KEYCHAIN" $existing

security import "$CERT_PATH" -k "$KEYCHAIN" -P "$APPLE_CERTIFICATE_PASSWORD" \
  -T /usr/bin/codesign -T /usr/bin/security -T /usr/bin/productbuild
security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$KEYCHAIN_PASSWORD" "$KEYCHAIN"

echo "Imported signing identity into $KEYCHAIN"
security find-identity -v -p codesigning "$KEYCHAIN"
