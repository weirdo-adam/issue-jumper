#!/usr/bin/env bash
set -euo pipefail

max_lines="${MAX_FILE_LINES:-500}"
status=0

is_checked_file() {
  case "$1" in
    *.lock | *.md | *.rs | *.sh | *.svg | *.toml | *.yaml | *.yml) return 0 ;;
    .editorconfig | .gitignore | LICENSE | Makefile) return 0 ;;
    *) return 1 ;;
  esac
}

while IFS= read -r -d '' file; do
  if [[ ! -f "$file" ]]; then
    continue
  fi
  if ! is_checked_file "$file"; then
    continue
  fi

  line_count="$(wc -l < "$file" | tr -d '[:space:]')"
  if (( line_count > max_lines )); then
    printf '%s has %s lines; max is %s\n' "$file" "$line_count" "$max_lines" >&2
    status=1
  fi
done < <(git ls-files -z --cached --others --exclude-standard)

if (( status != 0 )); then
  echo "File line limit failed." >&2
  exit "$status"
fi

echo "File line limit passed (max ${max_lines} lines)."
