#!/usr/bin/env bash
set -Eeuo pipefail

raw_base_was_set="${AISAAS_RAW_BASE+x}"
AISAAS_REPO="${AISAAS_REPO:-longxingze0925/AiSaaS}"
AISAAS_REF="${AISAAS_REF:-main}"
AISAAS_RAW_BASE="${AISAAS_RAW_BASE:-https://raw.githubusercontent.com/${AISAAS_REPO}/${AISAAS_REF}}"

script_dir=""
if [[ -n "${BASH_SOURCE[0]:-}" && -f "${BASH_SOURCE[0]}" ]]; then
  script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd -P || true)"
fi

if [[ -n "$script_dir" && -f "$script_dir/aisaasctl.sh" ]]; then
  exec bash "$script_dir/aisaasctl.sh"
fi

tmp_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

curl -fsSL "$AISAAS_RAW_BASE/ops/aisaasctl.sh" -o "$tmp_dir/aisaasctl.sh"
exec bash "$tmp_dir/aisaasctl.sh"
