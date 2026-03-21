#!/bin/bash
#
# release.sh — archinstall-rs
#
# Bumps Cargo.toml, refreshes Cargo.lock, commits, creates an annotated tag, and
# pushes to origin to trigger .github/workflows/release.yml.
#
# Usage:
#   ./scripts/release.sh [X.Y.Z]
#
# CI reads release notes from (first match):
#   Documents/RELEASE_vX.Y.Z.md
#   Documents/release_vX.Y.Z.md
#   Release-docs/RELEASE_vX.Y.Z.md
#   or the annotated tag message
#

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_info() { echo -e "${BLUE}ℹ${NC} $1"; }
print_success() { echo -e "${GREEN}✓${NC} $1"; }
print_warning() { echo -e "${YELLOW}⚠${NC} $1"; }
print_error() { echo -e "${RED}✗${NC} $1"; }

command_exists() { command -v "$1" >/dev/null 2>&1; }

print_info "Checking tools..."
if ! command_exists git; then
  print_error "git is not installed."
  exit 1
fi
print_success "git is installed"

GH_AVAILABLE=false
if command_exists gh; then
  GH_AVAILABLE=true
  print_success "GitHub CLI (gh) is available"
else
  print_warning "gh not installed (optional). https://cli.github.com/"
fi

if [ $# -eq 0 ]; then
  read -r -p "Version (X.Y.Z, no v prefix): " VERSION_INPUT
  if [ -z "$VERSION_INPUT" ]; then
    print_error "Version cannot be empty"
    exit 1
  fi
else
  VERSION_INPUT="$1"
fi

VERSION_INPUT="${VERSION_INPUT#v}"
if [[ ! "$VERSION_INPUT" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  print_error "Invalid version: $VERSION_INPUT (expected X.Y.Z)"
  exit 1
fi

VERSION="v$VERSION_INPUT"
print_success "Version: $VERSION"

if ! git rev-parse --git-dir >/dev/null 2>&1; then
  print_error "Not a git repository"
  exit 1
fi

if [ -n "$(git status --porcelain 2>/dev/null)" ]; then
  TRACKED=$(git status --porcelain 2>/dev/null | grep -v '^??' || true)
  if [ -n "$TRACKED" ]; then
    print_warning "Uncommitted changes in tracked files"
    read -r -p "Continue anyway? (y/N) " -n 1 -r
    echo
    [[ $REPLY =~ ^[Yy]$ ]] || exit 0
  fi
fi

if ! git remote get-url origin >/dev/null 2>&1; then
  print_error "No remote 'origin'"
  exit 1
fi

REMOTE_URL=$(git remote get-url origin)
REPO_NAME=""
if [[ "$REMOTE_URL" =~ github\.com[:/]([^/]+/[^/]+) ]]; then
  REPO_NAME="${BASH_REMATCH[1]}"
  REPO_NAME="${REPO_NAME%.git}"
fi

RELEASE_EXISTS=false
if [ "$GH_AVAILABLE" = true ] && [ -n "$REPO_NAME" ]; then
  if gh release view "$VERSION" --repo "$REPO_NAME" >/dev/null 2>&1; then
    RELEASE_EXISTS=true
    print_warning "Release $VERSION already exists on GitHub"
  fi
fi

TAG_EXISTS_LOCAL=false
TAG_EXISTS_REMOTE=false
git rev-parse "$VERSION" >/dev/null 2>&1 && TAG_EXISTS_LOCAL=true
git ls-remote --tags origin 2>&1 | grep -q "refs/tags/$VERSION$" && TAG_EXISTS_REMOTE=true

if [ "$RELEASE_EXISTS" = true ] || [ "$TAG_EXISTS_LOCAL" = true ] || [ "$TAG_EXISTS_REMOTE" = true ]; then
  print_warning "Removing existing GitHub release and/or tags for $VERSION"
  if [ "$RELEASE_EXISTS" = true ] && [ "$GH_AVAILABLE" = true ]; then
    gh release delete "$VERSION" --repo "$REPO_NAME" --yes >/dev/null 2>&1 || true
  elif [ "$RELEASE_EXISTS" = true ]; then
    print_warning "Delete release manually if needed: https://github.com/$REPO_NAME/releases"
  fi
  [ "$TAG_EXISTS_REMOTE" = true ] && git push origin ":refs/tags/$VERSION" >/dev/null 2>&1 || true
  [ "$TAG_EXISTS_LOCAL" = true ] && git tag -d "$VERSION" >/dev/null 2>&1 || true
fi

CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
print_info "Branch: $CURRENT_BRANCH"
if [[ "$CURRENT_BRANCH" != "main" && "$CURRENT_BRANCH" != "master" ]]; then
  print_warning "Not on main/master"
  read -r -p "Continue? (y/N) " -n 1 -r
  echo
  [[ $REPLY =~ ^[Yy]$ ]] || exit 0
fi

if [ -z "$REPO_NAME" ]; then
  print_error "Could not parse GitHub repo from origin URL"
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CARGO_TOML="$PROJECT_ROOT/Cargo.toml"

if [ ! -f "$CARGO_TOML" ]; then
  print_error "Missing $CARGO_TOML"
  exit 1
fi

print_info "Setting Cargo.toml version to $VERSION_INPUT..."
if [[ "$OSTYPE" == darwin* ]]; then
  sed -i '' "s/^version = \"[^\"]*\"/version = \"$VERSION_INPUT\"/" "$CARGO_TOML"
else
  sed -i "s/^version = \"[^\"]*\"/version = \"$VERSION_INPUT\"/" "$CARGO_TOML"
fi

if command_exists cargo; then
  (cd "$PROJECT_ROOT" && cargo generate-lockfile)
  print_success "Cargo.lock refreshed"
else
  print_warning "cargo not found; run 'cargo generate-lockfile' before pushing if CI uses --locked"
fi

DOC_RELEASE="$PROJECT_ROOT/Documents/RELEASE_${VERSION}.md"
DOC_ALT="$PROJECT_ROOT/Documents/release_v${VERSION_INPUT}.md"

RELEASE_NOTES="Release $VERSION"
if [ -f "$DOC_RELEASE" ]; then
  RELEASE_NOTES=$(cat "$DOC_RELEASE")
  print_success "Tag message from Documents/RELEASE_${VERSION}.md"
elif [ -f "$DOC_ALT" ]; then
  RELEASE_NOTES=$(cat "$DOC_ALT")
  print_success "Tag message from Documents/release_v${VERSION_INPUT}.md"
fi

git add "$CARGO_TOML"
[ -f "$PROJECT_ROOT/Cargo.lock" ] && git add "$PROJECT_ROOT/Cargo.lock"

[ -f "$DOC_RELEASE" ] && git add "$DOC_RELEASE"
[ -f "$DOC_ALT" ] && git add "$DOC_ALT"

if [ -z "$(git diff --cached --name-only 2>/dev/null)" ]; then
  print_error "Nothing to commit"
  exit 1
fi

print_info "Committing version bump (and release notes if staged)..."
git commit -m "chore: release $VERSION_INPUT"
git push origin "$CURRENT_BRANCH"
print_success "Pushed to $CURRENT_BRANCH"

echo ""
print_warning "Create and push tag $VERSION? This starts the release workflow."
read -r -p "Continue? (y/N) " -n 1 -r
echo
[[ $REPLY =~ ^[Yy]$ ]] || exit 0

TEMP_NOTES=$(mktemp)
printf '%s' "$RELEASE_NOTES" > "$TEMP_NOTES"
git tag -a "$VERSION" -F "$TEMP_NOTES"
rm -f "$TEMP_NOTES"

git push origin "$VERSION"
print_success "Tag pushed"

echo ""
print_success "Release workflow triggered."
print_info "Actions: https://github.com/$REPO_NAME/actions"
print_info "Release: https://github.com/$REPO_NAME/releases/tag/$VERSION"
echo ""
print_info "On the Arch ISO after the workflow finishes:"
echo "  curl -fsSL https://github.com/$REPO_NAME/releases/download/$VERSION/install.sh | bash"
