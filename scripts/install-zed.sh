#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_dir="$(cd "$script_dir/.." && pwd)"
key="alt-j"
force=1
print=0

usage() {
  cat <<'USAGE'
Usage: scripts/install-zed.sh [--key <key>] [--force] [--no-force] [--print]

Build issue-jumper from source and install the Zed task/keymap integration.

Options:
  --key <key>  Keybinding to install. Defaults to alt-j.
  --force      Replace an existing binding for the selected key. Default.
  --no-force   Fail instead of replacing an existing foreign binding.
  --print      Print the generated tasks/keymap snippets instead of writing.
  -h, --help   Show this help.
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --key)
      if [[ $# -lt 2 ]]; then
        echo "install-zed.sh: --key requires a value" >&2
        exit 2
      fi
      key="$2"
      shift 2
      ;;
    --force)
      force=1
      shift
      ;;
    --no-force)
      force=0
      shift
      ;;
    --print)
      print=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "install-zed.sh: unknown argument $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

cd "$repo_dir"
cargo build --release

binary="$repo_dir/target/release/issue-jumper"
args=("install-zed" "--key" "$key")
if [[ "$force" -eq 1 ]]; then
  args+=("--force")
fi
if [[ "$print" -eq 1 ]]; then
  args+=("--print")
fi

"$binary" "${args[@]}"
