#!/usr/bin/env bash
set -euo pipefail

# Explicit native SDK provisioning. This script is never called by Cargo/build.rs.
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
lock_file="$repo_root/tools/steam-audio-sdk.lock"
cache_root="${STEAMAUDIO_CACHE_DIR:-$repo_root/target/native/steam-audio}"
sdk_commit="$(sed -n 's/.*"commit": "\([^"]*\)".*/\1/p' "$lock_file")"
sdk_repo="$(sed -n 's/.*"repository": "\([^"]*\)".*/\1/p' "$lock_file")"
artifact="$cache_root/$sdk_commit/lib/linux-x64/libphonon.so"
manifest="$cache_root/$sdk_commit/build-inputs.txt"
source_dir="${STEAMAUDIO_SOURCE_DIR:-$cache_root/source-$sdk_commit}"

command -v cmake >/dev/null || { echo "cmake is required" >&2; exit 1; }
command -v c++ >/dev/null || { echo "a C++ compiler is required" >&2; exit 1; }

input_fingerprint() {
  {
    cat "$lock_file"
    sha256sum "$repo_root/tools/build_steam_audio_sdk.sh" | cut -d ' ' -f 1
    sha256sum "$(readlink -f "$(command -v cmake)")" "$(readlink -f "$(command -v c++)")"
    cmake --version | head -n 1
    c++ --version | head -n 1
    printf 'CC=%s\nCXX=%s\nCFLAGS=%s\nCXXFLAGS=%s\nLDFLAGS=%s\n' \
      "${CC:-}" "${CXX:-}" "${CFLAGS:-}" "${CXXFLAGS:-}" "${LDFLAGS:-}"
    printf '%s\n' 'STEAMAUDIO_ENABLE_IPP=FALSE' 'STEAMAUDIO_ENABLE_EMBREE=FALSE' \
      'STEAMAUDIO_ENABLE_RADEONRAYS=FALSE' 'STEAMAUDIO_ENABLE_TAN=FALSE'
  } | sha256sum | cut -d ' ' -f 1
}

fingerprint="$(input_fingerprint)"
if [[ "${1:-}" != "--force" && -f "$artifact" && -f "$manifest" ]] && \
   grep -qx "input_sha256=$fingerprint" "$manifest"; then
  expected="$(sed -n 's/^artifact_sha256=//p' "$manifest")"
  actual="$(sha256sum "$artifact" | cut -d ' ' -f 1)"
  if [[ "$actual" == "$expected" ]]; then
    echo "Steam Audio $sdk_commit is up to date; skipping SDK rebuild." >&2
    printf '%s\n' "$(dirname "$artifact")"
    exit 0
  fi
fi

if [[ ! -d "$source_dir/.git" ]]; then
  mkdir -p "$(dirname "$source_dir")"
  git clone --filter=blob:none "$sdk_repo" "$source_dir"
fi
if ! git -C "$source_dir" cat-file -e "$sdk_commit^{commit}" 2>/dev/null; then
  git -C "$source_dir" fetch --tags --force origin
fi
actual_commit="$(git -C "$source_dir" rev-parse "$sdk_commit^{commit}")"
[[ "$actual_commit" == "$sdk_commit" ]] || { echo "SDK commit mismatch" >&2; exit 1; }
git -C "$source_dir" checkout --detach "$sdk_commit"
actual_manifest="$(sha256sum "$source_dir/core/build/dependencies.json" | cut -d ' ' -f 1)"
expected_manifest="$(sed -n 's/.*"dependency_manifest_sha256": "\([^"]*\)".*/\1/p' "$lock_file")"
[[ "$actual_manifest" == "$expected_manifest" ]] || { echo "Dependency manifest hash mismatch" >&2; exit 1; }

# Steam Audio 4.8.1 predates the current CMake policy version and modern
# GCC warnings-as-errors. Apply compatibility flags only inside this opt-in build.
mkdir -p "$cache_root/cmake-compat" "$cache_root"
cat > "$cache_root/cmake-compat/cmake" <<'WRAPPER'
#!/usr/bin/env bash
export CMAKE_POLICY_VERSION_MINIMUM=3.5
export CXXFLAGS="${CXXFLAGS:+$CXXFLAGS }-Wno-error=unused-but-set-variable"
exec /usr/bin/cmake "$@"
WRAPPER
chmod +x "$cache_root/cmake-compat/cmake"

(cd "$source_dir/core/build" && \
  for dependency in flatbuffers zlib mysofa pffft; do \
    PATH="$cache_root/cmake-compat:$PATH" python3 get_dependencies.py \
      --platform linux --architecture x64 --dependency "$dependency"; \
  done && \
  PATH="$cache_root/cmake-compat:$PATH" python3 build.py \
    --platform linux --architecture x64 --configuration release \
    --minimal --operation ci_build)

source_artifact="$source_dir/core/bin/lib/linux-x64/libphonon.so"
[[ -f "$source_artifact" ]] || { echo "Steam Audio build did not produce libphonon.so" >&2; exit 1; }
mkdir -p "$(dirname "$artifact")"
cp "$source_artifact" "$artifact"
artifact_hash="$(sha256sum "$artifact" | cut -d ' ' -f 1)"
{
  printf 'input_sha256=%s\n' "$fingerprint"
  printf 'artifact_sha256=%s\n' "$artifact_hash"
} > "$manifest"
printf 'Built Steam Audio %s (%s)\n' "$sdk_commit" "$artifact_hash" >&2
printf '%s\n' "$(dirname "$artifact")"
