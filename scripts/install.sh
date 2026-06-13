#!/usr/bin/env sh
set -eu

repo="${ISSUE_JUMPER_REPO:-weirdo-adam/issue-jumper}"
version="${VERSION:-}"
key="${KEY:-alt alt}"
install_zed="${INSTALL_ZED:-1}"
force=1
uninstall=0

if [ -n "${INSTALL_DIR:-}" ]; then
  install_dir="$INSTALL_DIR"
else
  home="${HOME:-}"
  if [ -z "$home" ]; then
    echo "issue-jumper install: HOME is not set; pass --install-dir." >&2
    exit 1
  fi
  install_dir="$home/.local/bin"
fi

usage() {
  cat <<'USAGE'
Usage: scripts/install.sh [options]
       scripts/install.sh --uninstall [--install-dir <dir>]

Download a release archive, install issue-jumper, and install the Zed integration.
Prebuilt release archives currently support Apple Silicon macOS and Linux x64.

Options:
  --version <tag>      Release tag to install. Defaults to latest.
  --install-dir <dir>  Install directory. Defaults to ~/.local/bin.
  --key <key>          Zed keybinding. Defaults to alt alt.
  --force              Replace an existing Zed binding for the selected key. Default.
  --no-force           Fail instead of replacing an existing foreign Zed binding.
  --no-zed             Install only the CLI.
  --uninstall          Remove issue-jumper from the install directory and exit.
  --repo <owner/name>  GitHub repository. Defaults to weirdo-adam/issue-jumper.
  -h, --help           Show this help.

Environment:
  VERSION              Release tag to install.
  INSTALL_DIR          Install directory.
  KEY                  Zed keybinding.
  INSTALL_ZED=0        Install only the CLI.
  ISSUE_JUMPER_REPO    GitHub repository.
USAGE
}

die() {
  echo "issue-jumper install: $*" >&2
  exit 1
}

homebrew_binary() {
  command -v brew >/dev/null 2>&1 || return 0

  brew_prefix="$(brew --prefix 2>/dev/null || true)"
  [ -n "$brew_prefix" ] || return 0

  brew_binary="$brew_prefix/bin/issue-jumper"
  [ -x "$brew_binary" ] || return 0

  printf '%s\n' "$brew_binary"
}

warn_if_homebrew_install_exists() {
  brew_binary="$(homebrew_binary)"
  [ -n "$brew_binary" ] || return 0

  target_binary="$install_dir/issue-jumper"
  [ "$target_binary" = "$brew_binary" ] && return 0

  {
    echo "issue-jumper install: detected Homebrew issue-jumper at $brew_binary"
    echo "issue-jumper install: this script installs to $target_binary; keeping both can make PATH or Zed use a different copy than Homebrew upgrades."
    echo "issue-jumper install: to remove the manual copy, run this installer again with --uninstall"
    echo "issue-jumper install: for a Homebrew-managed setup, run: $brew_binary install-zed --force"
  } >&2
}

uninstall_manual_install() {
  target_binary="$install_dir/issue-jumper"

  if [ -d "$target_binary" ]; then
    die "$target_binary is a directory; remove it manually"
  fi

  if [ -e "$target_binary" ] || [ -L "$target_binary" ]; then
    rm -f "$target_binary"
    echo "Removed issue-jumper from $target_binary"
  else
    echo "No issue-jumper binary found at $target_binary"
  fi

  brew_binary="$(homebrew_binary)"
  if [ -n "$brew_binary" ]; then
    echo "Homebrew issue-jumper is available at $brew_binary"
    echo "To point Zed at Homebrew, run: $brew_binary install-zed --force"
  else
    echo "To install with Homebrew, run: brew install weirdo-adam/tap/issue-jumper"
  fi
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --version)
      [ "$#" -ge 2 ] || die "--version requires a value"
      version="$2"
      shift 2
      ;;
    --install-dir)
      [ "$#" -ge 2 ] || die "--install-dir requires a value"
      install_dir="$2"
      shift 2
      ;;
    --key)
      [ "$#" -ge 2 ] || die "--key requires a value"
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
    --no-zed)
      install_zed=0
      shift
      ;;
    --uninstall)
      uninstall=1
      shift
      ;;
    --repo)
      [ "$#" -ge 2 ] || die "--repo requires a value"
      repo="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      die "unknown argument $1"
      ;;
  esac
done

if [ "$uninstall" -eq 1 ]; then
  uninstall_manual_install
  exit 0
fi

warn_if_homebrew_install_exists

download_stdout() {
  tmp_base="${TMPDIR:-/tmp}"
  tmp_file="$(mktemp "$tmp_base/issue-jumper.download.XXXXXX")" \
    || die "could not create temporary file"

  download_file "$1" "$tmp_file"
  cat "$tmp_file"
  rm -f "$tmp_file"
}

download_file() {
  attempted=0

  if command -v curl >/dev/null 2>&1; then
    attempted=1
    if curl -fsSL --connect-timeout 15 --retry 2 --retry-delay 1 -o "$2" "$1"; then
      return 0
    fi
  fi

  if command -v wget >/dev/null 2>&1; then
    attempted=1
    if wget -q --timeout=15 --tries=2 -O "$2" "$1"; then
      return 0
    fi
  fi

  [ "$attempted" -eq 1 ] || die "curl or wget is required"
  die "failed to download $1"
}

detect_target() {
  os="$(uname -s 2>/dev/null || true)"
  arch="$(uname -m 2>/dev/null || true)"

  case "$os:$arch" in
    Darwin:arm64|Darwin:aarch64) echo "aarch64-apple-darwin" ;;
    Darwin:x86_64)
      die "no prebuilt release archive is published for macOS x86_64; build locally with scripts/package-release.sh on that host"
      ;;
    Linux:x86_64|Linux:amd64) echo "x86_64-unknown-linux-gnu" ;;
    *)
      die "unsupported platform $os/$arch"
      ;;
  esac
}

if [ -z "$version" ]; then
  version="$(
    download_stdout "https://api.github.com/repos/$repo/releases/latest" \
      | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' \
      | head -n 1
  )"
  [ -n "$version" ] || die "could not resolve latest release tag"
fi

target="$(detect_target)"
archive="issue-jumper-${version}-${target}.tar.gz"
archive_url="https://github.com/$repo/releases/download/$version/$archive"
tmp_base="${TMPDIR:-/tmp}"
tmp_dir="$(mktemp -d "$tmp_base/issue-jumper.XXXXXX")"
trap 'rm -rf "$tmp_dir"' EXIT INT HUP TERM

download_file "$archive_url" "$tmp_dir/$archive"
tar -xzf "$tmp_dir/$archive" -C "$tmp_dir"

binary="$tmp_dir/issue-jumper-${version}-${target}/issue-jumper"
[ -x "$binary" ] || die "release archive does not contain issue-jumper"

mkdir -p "$install_dir"
install -m 0755 "$binary" "$install_dir/issue-jumper"

echo "Installed issue-jumper $version to $install_dir/issue-jumper"

if [ "$install_zed" != "0" ]; then
  if [ "$force" -eq 1 ]; then
    "$install_dir/issue-jumper" install-zed --key "$key" --force
  else
    "$install_dir/issue-jumper" install-zed --key "$key"
  fi
else
  echo "Skipped Zed integration."
fi
