#!/bin/sh

set -eu

REPOSITORY="${VULCANUM_REPOSITORY:-EzyGang/vulcanum}"
INSTALL_DIR="${VULCANUM_INSTALL_DIR:-${HOME}/.local/bin}"
VERSION="${VULCANUM_VERSION:-latest}"

require_command() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "error: required command '$1' was not found" >&2
        exit 1
    fi
}

print_path_instructions() {
    path_value=$INSTALL_DIR
    if [ "$INSTALL_DIR" = "${HOME}/.local/bin" ]; then
        path_value='$HOME/.local/bin'
    fi

    shell_name=${SHELL##*/}
    echo

    case "$shell_name" in
        fish)
            echo "Add it to PATH for current and future fish sessions:"
            printf '  fish_add_path "%s"\n' "$path_value"
            ;;
        bash|zsh)
            profile='$HOME/.bashrc'
            if [ "$shell_name" = "zsh" ]; then
                profile='$HOME/.zshrc'
            fi

            echo "Add it to PATH for this shell:"
            printf '  export PATH="%s:$PATH"\n' "$path_value"
            echo
            echo "Persist it for future ${shell_name} sessions:"
            echo "  echo 'export PATH=\"${path_value}:\$PATH\"' >> \"${profile}\""
            echo "  . \"${profile}\""
            ;;
        *)
            echo "Add it to PATH for this shell:"
            printf '  export PATH="%s:$PATH"\n' "$path_value"
            echo
            echo "Add the same export command to your shell profile to persist it."
            ;;
    esac
}

resolve_target() {
    os=$(uname -s)
    arch=$(uname -m)

    case "$os" in
        Linux) os="unknown-linux-gnu" ;;
        Darwin) os="apple-darwin" ;;
        *)
            echo "error: unsupported operating system: $os" >&2
            exit 1
            ;;
    esac

    case "$arch" in
        x86_64|amd64) arch="x86_64" ;;
        arm64|aarch64) arch="aarch64" ;;
        *)
            echo "error: unsupported architecture: $arch" >&2
            exit 1
            ;;
    esac

    printf '%s-%s\n' "$arch" "$os"
}

download() {
    download_url=$1
    download_output=$2

    if command -v curl >/dev/null 2>&1; then
        curl --fail --location --silent --show-error --output "$download_output" "$download_url"
    elif command -v wget >/dev/null 2>&1; then
        wget --quiet --output-document="$download_output" "$download_url"
    else
        echo "error: either curl or wget is required" >&2
        exit 1
    fi
}

resolve_latest_tag() {
    release_metadata_path=$1
    download "https://api.github.com/repos/${REPOSITORY}/releases?per_page=1" "$release_metadata_path"
    latest_tag=$(sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$release_metadata_path")

    if [ -z "$latest_tag" ]; then
        echo "error: no published Vulcanum release was found" >&2
        exit 1
    fi

    printf '%s\n' "$latest_tag"
}

verify_checksum() {
    checksum_archive=$1
    checksum_file=$2
    expected_checksum=$(awk '{print $1}' "$checksum_file")

    if command -v sha256sum >/dev/null 2>&1; then
        actual_checksum=$(sha256sum "$checksum_archive" | awk '{print $1}')
    elif command -v shasum >/dev/null 2>&1; then
        actual_checksum=$(shasum -a 256 "$checksum_archive" | awk '{print $1}')
    else
        echo "error: sha256sum or shasum is required to verify the download" >&2
        exit 1
    fi

    if [ "$actual_checksum" != "$expected_checksum" ]; then
        echo "error: checksum verification failed" >&2
        exit 1
    fi
}

require_command uname
require_command tar
require_command awk
require_command sed

tmp_dir=$(mktemp -d 2>/dev/null || mktemp -d -t vulcanum)
trap 'rm -rf "$tmp_dir"' EXIT HUP INT TERM

target=$(resolve_target)
archive_name="vulcanum-${target}.tar.gz"

if [ "$VERSION" = "latest" ]; then
    tag=$(resolve_latest_tag "${tmp_dir}/release.json")
else
    case "$VERSION" in
        v*) tag=$VERSION ;;
        *) tag="v${VERSION}" ;;
    esac
fi

release_url="https://github.com/${REPOSITORY}/releases/download/${tag}"

echo "Downloading Vulcanum ${tag} for ${target}..."
download "${release_url}/${archive_name}" "${tmp_dir}/${archive_name}"
download "${release_url}/${archive_name}.sha256" "${tmp_dir}/${archive_name}.sha256"
verify_checksum "${tmp_dir}/${archive_name}" "${tmp_dir}/${archive_name}.sha256"
tar -xzf "${tmp_dir}/${archive_name}" -C "$tmp_dir"

mkdir -p "$INSTALL_DIR"
install -m 0755 "${tmp_dir}/vulcanum" "${INSTALL_DIR}/vulcanum"
install -m 0755 "${tmp_dir}/vulcanum-server" "${INSTALL_DIR}/vulcanum-server"

echo "Installed vulcanum and vulcanum-server to ${INSTALL_DIR}"
case ":${PATH}:" in
    *":${INSTALL_DIR}:"*) ;;
    *) print_path_instructions ;;
esac
