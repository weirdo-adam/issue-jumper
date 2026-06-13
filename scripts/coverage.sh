#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
target_dir="$root_dir/target/llvm-cov-target-production"
profile_dir="$(mktemp -d "$root_dir/target/llvm-cov-profraw-production.XXXXXX")"
profdata="$profile_dir/coverage.profdata"

if command -v llvm-cov >/dev/null 2>&1; then
  llvm_cov="llvm-cov"
  llvm_profdata="llvm-profdata"
elif command -v xcrun >/dev/null 2>&1; then
  llvm_cov="xcrun llvm-cov"
  llvm_profdata="xcrun llvm-profdata"
else
  echo "llvm-cov or xcrun is required for coverage" >&2
  exit 1
fi

(
  cd "$root_dir"
  CARGO_TARGET_DIR="$target_dir" \
    CARGO_INCREMENTAL=0 \
    RUSTFLAGS="-Cinstrument-coverage" \
    LLVM_PROFILE_FILE="$profile_dir/issue-jumper-%p-%m.profraw" \
    cargo test --all-targets
)

$llvm_profdata merge -sparse "$profile_dir"/*.profraw -o "$profdata"

object_files=()
while IFS= read -r object_file; do
  dep_file="$object_file.d"
  if [[ -f "$dep_file" ]] && grep -q ': src/main.rs$' "$dep_file"; then
    continue
  fi
  object_files+=("$object_file")
done < <(
  find "$target_dir/debug/deps" \
    -maxdepth 1 \
    -type f \
    -perm -111 \
    ! -name '*.d' \
    ! -name '*.dylib' \
    ! -name '*.o' \
    | sort
)

if [[ -x "$target_dir/debug/issue-jumper" ]]; then
  object_files+=("$target_dir/debug/issue-jumper")
fi

if [[ "${#object_files[@]}" -eq 0 ]]; then
  echo "no coverage objects found" >&2
  exit 1
fi

primary_object="${object_files[0]}"
extra_objects=()
for ((index = 1; index < ${#object_files[@]}; index++)); do
  extra_objects+=("-object" "${object_files[$index]}")
done

report="$(
  $llvm_cov report \
    --instr-profile="$profdata" \
    --ignore-filename-regex='/.cargo/registry|/rustc/|target/|(^|/)tests/|src/.*/tests\.rs|/opt/homebrew/Cellar/rust|src/browser/platform/(windows|unix)\.rs|src/zed/platform/windows\.rs' \
    "$primary_object" \
    "${extra_objects[@]}"
)"

printf '%s\n' "$report"

line_coverage="$(printf '%s\n' "$report" | awk '/^TOTAL / { print $10 }')"
if [[ "$line_coverage" != "100.00%" ]]; then
  echo "production code line coverage must be 100.00%; got $line_coverage" >&2
  exit 1
fi
