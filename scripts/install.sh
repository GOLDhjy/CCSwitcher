#!/usr/bin/env bash
set -euo pipefail

GITHUB_REPO="${CCSWITCHER_GITHUB_REPO:-GOLDhjy/CCSwitcher}"
REPO_URL="${CCSWITCHER_REPO_URL:-https://github.com/GOLDhjy/CCSwitcher.git}"
VERSION="${CCSWITCHER_VERSION:-latest}"
MODE="binary"
BIN_DIR="${INSTALL_BIN_DIR:-$HOME/.local/bin}"
TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/ccswitcher-target}"
SCRIPT_SOURCE="${BASH_SOURCE[0]-$0}"
ROOT_DIR=""
TMP_DIRS=()

cleanup_tmp_dirs() {
  local dir
  for dir in "${TMP_DIRS[@]-}"; do
    [[ -n "$dir" && -d "$dir" ]] && rm -rf "$dir"
  done
}

register_tmp_dir() {
  TMP_DIRS+=("$1")
}

trap cleanup_tmp_dirs EXIT

if [[ "$SCRIPT_SOURCE" != "bash" && "$SCRIPT_SOURCE" != "-" ]]; then
  ROOT_DIR="$(cd "$(dirname "$SCRIPT_SOURCE")/.." && pwd 2>/dev/null || true)"
fi

usage() {
  cat <<'USAGE'
Usage: bash scripts/install.sh [options]

Options:
  --version <tag>   Install a specific release tag (e.g. v0.1.0). Default: latest
  --source          Build from source instead of downloading prebuilt binary
  --repo <git-url>  Source repository URL used with --source
USAGE
}

require_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "Missing required command: $cmd"
    exit 1
  fi
}

detect_target() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"
  case "$os" in
    Darwin)
      case "$arch" in
        arm64|aarch64) echo "aarch64-apple-darwin" ;;
        x86_64) echo "x86_64-apple-darwin" ;;
        *) return 1 ;;
      esac
      ;;
    Linux)
      case "$arch" in
        x86_64) echo "x86_64-unknown-linux-gnu" ;;
        *) return 1 ;;
      esac
      ;;
    *)
      return 1
      ;;
  esac
}

latest_tag() {
  local resolved
  resolved="$(curl -fsSL -o /dev/null -w '%{url_effective}' "https://github.com/$GITHUB_REPO/releases/latest")"
  printf '%s\n' "${resolved##*/}"
}

print_success() {
  echo
  echo "Installed binary: $BIN_DIR/ccswitcher"
  echo "Installed slash command via: $BIN_DIR/ccswitcher install"
  echo
  echo "If needed, add this to your shell rc:"
  echo "  export PATH=\"$BIN_DIR:\$PATH\""
}

install_from_binary() {
  require_cmd curl
  require_cmd tar

  local target tag asset checksum_asset base_url tmp_dir asset_file checksum_file expected actual
  target="$(detect_target)" || {
    echo "Unsupported platform for prebuilt binary. Use --source."
    return 1
  }

  if [[ "$VERSION" == "latest" ]]; then
    tag="$(latest_tag)"
    if [[ -z "$tag" ]]; then
      echo "Failed to detect latest release tag from GitHub."
      return 1
    fi
  else
    tag="$VERSION"
  fi

  asset="ccswitcher-${tag}-${target}.tar.gz"
  checksum_asset="${asset}.sha256"
  base_url="https://github.com/$GITHUB_REPO/releases/download/$tag"

  tmp_dir="$(mktemp -d)"
  register_tmp_dir "$tmp_dir"

  echo "Downloading release ${tag} (${target})..."
  asset_file="$tmp_dir/$asset"
  checksum_file="$tmp_dir/$checksum_asset"

  curl -fL --retry 3 --retry-delay 1 -o "$asset_file" "$base_url/$asset"
  curl -fL --retry 3 --retry-delay 1 -o "$checksum_file" "$base_url/$checksum_asset"

  expected="$(awk '{print $1}' "$checksum_file")"
  if [[ -z "$expected" ]]; then
    echo "Invalid checksum file: $checksum_asset"
    return 1
  fi

  if command -v shasum >/dev/null 2>&1; then
    actual="$(shasum -a 256 "$asset_file" | awk '{print $1}')"
  elif command -v sha256sum >/dev/null 2>&1; then
    actual="$(sha256sum "$asset_file" | awk '{print $1}')"
  else
    echo "Missing checksum tool (shasum or sha256sum)."
    return 1
  fi

  if [[ "$expected" != "$actual" ]]; then
    echo "Checksum mismatch for downloaded binary."
    return 1
  fi

  mkdir -p "$BIN_DIR"
  tar -xzf "$asset_file" -C "$tmp_dir"
  cp "$tmp_dir/ccswitcher" "$BIN_DIR/ccswitcher"
  chmod +x "$BIN_DIR/ccswitcher"
  "$BIN_DIR/ccswitcher" install

  print_success
}

install_from_source() {
  require_cmd cargo
  require_cmd git

  local source_root tmp_dir
  if [[ -n "$ROOT_DIR" && -f "$ROOT_DIR/Cargo.toml" ]]; then
    source_root="$ROOT_DIR"
  else
    tmp_dir="$(mktemp -d)"
    register_tmp_dir "$tmp_dir"
    echo "Cloning repository: $REPO_URL"
    git clone --depth 1 "$REPO_URL" "$tmp_dir/CCSwitcher"
    source_root="$tmp_dir/CCSwitcher"
  fi

  echo "Building ccswitcher from source..."
  CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="$TARGET_DIR" \
    cargo build --release --manifest-path "$source_root/Cargo.toml"

  mkdir -p "$BIN_DIR"
  cp "$TARGET_DIR/release/ccswitcher" "$BIN_DIR/ccswitcher"
  chmod +x "$BIN_DIR/ccswitcher"
  "$BIN_DIR/ccswitcher" install

  print_success
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --version)
      VERSION="${2:-}"
      shift 2
      ;;
    --source)
      MODE="source"
      shift
      ;;
    --repo)
      REPO_URL="${2:-$REPO_URL}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1"
      usage
      exit 1
      ;;
  esac
done

if [[ "$MODE" == "source" ]]; then
  install_from_source
else
  install_from_binary || {
    echo
    echo "Binary install failed. You can retry source build mode:"
    echo "  bash scripts/install.sh --source"
    exit 1
  }
fi
