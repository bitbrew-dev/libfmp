#!/usr/bin/env bash

set -euo pipefail

readonly crate_name="libfmp"
readonly max_kib="${LIBFMP_CRATE_MAX_KIB:-512}"

if ! [[ "$max_kib" =~ ^[1-9][0-9]*$ ]]; then
    echo "LIBFMP_CRATE_MAX_KIB must be a positive integer" >&2
    exit 2
fi

if ! command -v python3 >/dev/null 2>&1; then
    echo "python3 is required to read Cargo package metadata" >&2
    exit 2
fi

package_list="$(mktemp)"
trap 'rm -f "$package_list"' EXIT

cargo package -p "$crate_name" --list --allow-dirty --quiet >"$package_list"

if grep -Eq '(^|/)(tests?|fixtures)/' "$package_list"; then
    echo "published crate contains a test or fixture directory:" >&2
    grep -E '(^|/)(tests?|fixtures)/' "$package_list" >&2
    exit 1
fi

cargo package -p "$crate_name" --allow-dirty --offline --quiet

metadata="$(cargo metadata --no-deps --format-version 1 --offline)"
target_directory="$(python3 -c 'import json, sys; print(json.load(sys.stdin)["target_directory"])' <<<"$metadata")"
workspace_version="$(python3 -c 'import json, sys; data = json.load(sys.stdin); print(next(package["version"] for package in data["packages"] if package["name"] == "libfmp"))' <<<"$metadata")"
artifact="${target_directory}/package/${crate_name}-${workspace_version}.crate"

if [[ -z "$target_directory" || -z "$workspace_version" || ! -f "$artifact" ]]; then
    echo "could not locate the packaged crate artifact" >&2
    exit 1
fi

size_bytes="$(wc -c <"$artifact" | tr -d '[:space:]')"
max_bytes="$((max_kib * 1024))"

if ((size_bytes > max_bytes)); then
    echo "${artifact} is ${size_bytes} bytes; budget is ${max_bytes} bytes (${max_kib} KiB)" >&2
    exit 1
fi

file_count="$(wc -l <"$package_list" | tr -d '[:space:]')"
size_kib="$(((size_bytes + 1023) / 1024))"
echo "${artifact}: ${file_count} files, ${size_kib} KiB compressed (budget: ${max_kib} KiB)"
