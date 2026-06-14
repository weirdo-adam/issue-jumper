#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_dir="$(cd "$script_dir/.." && pwd)"
target=""
version=""
out_dir="$repo_dir/dist"

usage() {
  cat <<'USAGE'
Usage: scripts/package-release.sh [--target <triple>] [--version <version>] [--out <dir>]

Build and package a local CLI release artifact. By default, the script packages
the current host target.

Examples:
  scripts/package-release.sh
  scripts/package-release.sh --version v0.1.1
  scripts/package-release.sh --target aarch64-apple-darwin --version v0.1.1
USAGE
}

host_target() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"
  case "$os:$arch" in
    Darwin:arm64) echo "aarch64-apple-darwin" ;;
    Darwin:x86_64) echo "x86_64-apple-darwin" ;;
    Linux:x86_64) echo "x86_64-unknown-linux-gnu" ;;
    *)
      echo "Unsupported host $os/$arch. Pass --target explicitly." >&2
      exit 2
      ;;
  esac
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --target)
      if [[ $# -lt 2 ]]; then
        echo "package-release.sh: --target requires a value" >&2
        exit 2
      fi
      target="$2"
      shift 2
      ;;
    --version)
      if [[ $# -lt 2 ]]; then
        echo "package-release.sh: --version requires a value" >&2
        exit 2
      fi
      version="$2"
      shift 2
      ;;
    --out)
      if [[ $# -lt 2 ]]; then
        echo "package-release.sh: --out requires a value" >&2
        exit 2
      fi
      out_dir="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "package-release.sh: unknown argument $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [[ -z "$target" ]]; then
  target="$(host_target)"
fi

if [[ -z "$version" ]]; then
  version="$(sed -n 's/^version = "\(.*\)"/v\1/p' "$repo_dir/Cargo.toml" | head -n 1)"
fi

if [[ ! "$version" =~ ^v[0-9]+[.][0-9]+[.][0-9]+$ ]]; then
  echo "package-release.sh: version must look like vX.Y.Z: $version" >&2
  exit 2
fi

if [[ ! "$target" =~ ^[A-Za-z0-9._-]+$ ]]; then
  echo "package-release.sh: target contains unsupported characters: $target" >&2
  exit 2
fi

case "$target" in
  *windows*) binary_name="issue-jumper.exe"; archive_ext="zip" ;;
  *) binary_name="issue-jumper"; archive_ext="tar.gz" ;;
esac

mkdir -p "$out_dir"
out_dir="$(cd "$out_dir" && pwd)"
package_name="issue-jumper-${version}-${target}"
package_dir="$out_dir/$package_name"

cd "$repo_dir"
cargo build --release --target "$target"

rm -rf "$package_dir"
mkdir -p "$package_dir"
cp "target/$target/release/$binary_name" "$package_dir/"
cp README.md README.zh-CN.md LICENSE "$package_dir/"

if [[ "$archive_ext" == "zip" ]]; then
  archive="$out_dir/$package_name.zip"
  rm -f "$archive"
  (cd "$out_dir" && zip -qr "$archive" "$package_name")
else
  archive="$out_dir/$package_name.tar.gz"
  rm -f "$archive"
  tar -C "$out_dir" -czf "$archive" "$package_name"
fi

echo "$archive"
