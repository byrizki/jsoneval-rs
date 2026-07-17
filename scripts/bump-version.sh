#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

usage() {
  cat <<EOF
Usage: $(basename "$0") [OPTIONS] <message> [<message2> ...]

Bump patch version; update CHANGELOG.md, package manifests, README, and examples.

OPTIONS
  -v, --version <ver>   Use an explicit version instead of auto-incrementing
  -h, --help            Show this help

EXAMPLES
  $(basename "$0") "[core] fix data mutation on get_schema_value"
  $(basename "$0") "[core] fix foo" "[RN] fix bar"
  $(basename "$0") -v 0.1.0 "[core] new feature"
EOF
  exit 0
}

die() { echo "error: $1" >&2; exit 1; }

EXPLICIT_VERSION=""
MESSAGES=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    -v|--version) EXPLICIT_VERSION="$2"; shift 2 ;;
    -h|--help)    usage ;;
    -*)           die "unknown option: $1" ;;
    *)            MESSAGES+=("$1"); shift ;;
  esac
done

if [[ ${#MESSAGES[@]} -eq 0 ]]; then
  if git -C "$REPO_ROOT" rev-parse '@{u}' &>/dev/null 2>&1; then
    RANGE="@{u}..HEAD"
    echo "No messages provided — reading unpushed commits"
  else
    LAST_TAG=$(git -C "$REPO_ROOT" describe --tags --abbrev=0 2>/dev/null || true)
    [[ -n "$LAST_TAG" ]] || die "no messages provided, no upstream tracking branch, and no git tags found"
    RANGE="$LAST_TAG..HEAD"
    echo "No messages provided — no upstream set, reading commits since $LAST_TAG"
  fi
  while IFS= read -r line; do
    MESSAGES+=("$line")
  done < <(git -C "$REPO_ROOT" log "$RANGE" --pretty=format:"%s" --no-merges \
    | grep -Ev '^(v[0-9]+\.[0-9]+\.[0-9]+|chore: bump version)')
  [[ ${#MESSAGES[@]} -gt 0 ]] || die "no commits found in range $RANGE"
fi

CARGO_TOML="$REPO_ROOT/Cargo.toml"
CURRENT_VERSION=$(grep -m1 '^version\s*=' "$CARGO_TOML" | sed 's/.*"\(.*\)".*/\1/')
[[ -n "$CURRENT_VERSION" ]] || die "could not read version from Cargo.toml"

if [[ -n "$EXPLICIT_VERSION" ]]; then
  NEW_VERSION="$EXPLICIT_VERSION"
else
  IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"
  NEW_VERSION="$MAJOR.$MINOR.$((PATCH + 1))"
fi

TODAY=$(date +%Y-%m-%d)

echo "Bumping: $CURRENT_VERSION → $NEW_VERSION  ($TODAY)"
echo ""

STAGED_FILES=()

sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" "$CARGO_TOML"
STAGED_FILES+=("$CARGO_TOML")
echo "  ✔  Cargo.toml"

cargo update --package json-eval-rs --quiet
STAGED_FILES+=("$REPO_ROOT/Cargo.lock")
echo "  ✔  Cargo.lock"

PACKAGE_FILES=(
  "bindings/npm/packages/common/package.json"
  "bindings/npm/packages/bundler/package.json"
  "bindings/npm/packages/node/package.json"
  "bindings/npm/packages/react-native/package.json"
  "bindings/npm/packages/vanilla/package.json"
  "bindings/npm/packages/webcore/package.json"
)

for PKG in "${PACKAGE_FILES[@]}"; do
  FULL_PATH="$REPO_ROOT/$PKG"
  if [[ -f "$FULL_PATH" ]]; then
    sed -i "s/\"version\": \"$CURRENT_VERSION\"/\"version\": \"$NEW_VERSION\"/" "$FULL_PATH"
    STAGED_FILES+=("$FULL_PATH")
    echo "  ✔  $PKG"
  else
    echo "  ⚠  skipped (not found): $PKG"
  fi
done

# Update version references in root README and npm examples.
VERSION_REFERENCE_FILES=(
  "README.md"
  "bindings/npm/package.json"
  "bindings/npm/examples/nextjs/package.json"
  "bindings/npm/examples/nodejs-benchmark/README.md"
  "bindings/npm/examples/rncli/package.json"
)

for FILE in "${VERSION_REFERENCE_FILES[@]}"; do
  FULL_PATH="$REPO_ROOT/$FILE"
  if [[ -f "$FULL_PATH" ]]; then
    sed -i "s/$CURRENT_VERSION/$NEW_VERSION/g" "$FULL_PATH"
    STAGED_FILES+=("$FULL_PATH")
    echo "  ✔  $FILE"
  else
    echo "  ⚠  skipped (not found): $FILE"
  fi
done

# Update C# binding csproj
CSPROJ="$REPO_ROOT/bindings/csharp/JsonEvalRs.csproj"
if [[ -f "$CSPROJ" ]]; then
  sed -i \
    "s|<Version>$CURRENT_VERSION</Version>|<Version>$NEW_VERSION</Version>|g; \
     s|<AssemblyVersion>$CURRENT_VERSION\.0</AssemblyVersion>|<AssemblyVersion>$NEW_VERSION.0</AssemblyVersion>|g; \
     s|<FileVersion>$CURRENT_VERSION\.0</FileVersion>|<FileVersion>$NEW_VERSION.0</FileVersion>|g; \
     s|<InformationalVersion>$CURRENT_VERSION</InformationalVersion>|<InformationalVersion>$NEW_VERSION</InformationalVersion>|g" \
    "$CSPROJ"
  STAGED_FILES+=("$CSPROJ")
  echo "  ✔  bindings/csharp/JsonEvalRs.csproj"
else
  echo "  ⚠  skipped (not found): bindings/csharp/JsonEvalRs.csproj"
fi

# Update version references in docs markdown files
while IFS= read -r DOC_FILE; do
  REL_PATH="${DOC_FILE#$REPO_ROOT/}"
  sed -i "s/json-eval-rs = \"$CURRENT_VERSION\"/json-eval-rs = \"$NEW_VERSION\"/g" "$DOC_FILE"
  STAGED_FILES+=("$DOC_FILE")
  echo "  ✔  $REL_PATH"
done < <(grep -rl "json-eval-rs = \"$CURRENT_VERSION\"" "$REPO_ROOT/docs" 2>/dev/null || true)

# Update version references in .github files (workflows, READMEs)
while IFS= read -r GH_FILE; do
  REL_PATH="${GH_FILE#$REPO_ROOT/}"
  sed -i "s/v$CURRENT_VERSION/v$NEW_VERSION/g; s/$CURRENT_VERSION/$NEW_VERSION/g" "$GH_FILE"
  STAGED_FILES+=("$GH_FILE")
  echo "  ✔  $REL_PATH"
done < <(grep -rl "$CURRENT_VERSION" "$REPO_ROOT/.github" 2>/dev/null || true)

CHANGELOG="$REPO_ROOT/CHANGELOG.md"

ENTRY="## [$NEW_VERSION] - $TODAY"$'\n'
for MSG in "${MESSAGES[@]}"; do
  ENTRY+=$'\n'"- $MSG"
done
ENTRY+=$'\n'

TEMP_FILE=$(mktemp)
{
  head -n 1 "$CHANGELOG"
  printf '\n'
  printf '%s\n' "$ENTRY"
  tail -n +2 "$CHANGELOG"
} > "$TEMP_FILE"
mv "$TEMP_FILE" "$CHANGELOG"

STAGED_FILES+=("$CHANGELOG")
echo "  ✔  CHANGELOG.md"

git -C "$REPO_ROOT" add "${STAGED_FILES[@]}"
echo ""
echo "Staged ${#STAGED_FILES[@]} file(s)."

echo ""
echo "Done. New version: $NEW_VERSION"
echo ""
echo "Next steps:"
echo "  cargo check"
echo "  git commit -m \"chore: bump version to $NEW_VERSION\""
echo "  git tag v$NEW_VERSION"
