#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_dir="$(cd "$script_dir/.." && pwd)"
repo="weirdo-adam/issue-jumper"
target=""
draft=0
prerelease=0

usage() {
  cat <<'USAGE'
Usage: scripts/publish-release.sh <tag> [--target <triple>] [--repo <owner/name>] [--draft] [--prerelease]

Build a local release package and upload it to GitHub Releases with gh.
The tag should include the leading v, for example v0.1.0.

Examples:
  scripts/publish-release.sh v0.1.0
  scripts/publish-release.sh v0.1.0 --target aarch64-apple-darwin
USAGE
}

if [[ $# -lt 1 ]]; then
  usage >&2
  exit 2
fi

case "$1" in
  -h|--help)
    usage
    exit 0
    ;;
esac

tag="$1"
shift

while [[ $# -gt 0 ]]; do
  case "$1" in
    --target)
      if [[ $# -lt 2 ]]; then
        echo "publish-release.sh: --target requires a value" >&2
        exit 2
      fi
      target="$2"
      shift 2
      ;;
    --repo)
      if [[ $# -lt 2 ]]; then
        echo "publish-release.sh: --repo requires a value" >&2
        exit 2
      fi
      repo="$2"
      shift 2
      ;;
    --draft)
      draft=1
      shift
      ;;
    --prerelease)
      prerelease=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "publish-release.sh: unknown argument $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

package_args=("--version" "$tag")
if [[ -n "$target" ]]; then
  package_args+=("--target" "$target")
fi

archive="$("$repo_dir/scripts/package-release.sh" "${package_args[@]}")"

if gh release view "$tag" --repo "$repo" >/dev/null 2>&1; then
  gh release upload "$tag" "$archive" --repo "$repo" --clobber
else
  create_args=("$tag" "$archive" "--repo" "$repo" "--title" "$tag" "--generate-notes")
  if [[ "$draft" -eq 1 ]]; then
    create_args+=("--draft")
  fi
  if [[ "$prerelease" -eq 1 ]]; then
    create_args+=("--prerelease")
  fi
  gh release create "${create_args[@]}"
fi

echo "Uploaded $archive to $repo release $tag"
