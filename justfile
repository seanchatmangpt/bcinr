@ensure-ggen:
    #!/usr/bin/env bash
    set -euo pipefail

    if ! command -v ggen &> /dev/null; then
        echo "ggen not found. Downloading v26.15.2 binary..."
        PLATFORM="$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m)"
        case "$PLATFORM" in
            linux-x86_64) ARCH="x86_64-unknown-linux-gnu" ;;
            linux-aarch64) ARCH="aarch64-unknown-linux-gnu" ;;
            darwin-x86_64) ARCH="x86_64-apple-darwin" ;;
            darwin-arm64) ARCH="aarch64-apple-darwin" ;;
            *) echo "Unsupported platform: $PLATFORM"; exit 1 ;;
        esac

        URL="https://github.com/seanchatmangpt/ggen/releases/download/v26.15.2/ggen-${ARCH}.tar.gz"
        mkdir -p "$HOME/.local/bin"
        curl --fail --silent --show-error --location "$URL" | tar xz -C "$HOME/.local/bin"
        export PATH="$HOME/.local/bin:$PATH"
    fi

    echo "ggen $(ggen --version)"

@sync: ensure-ggen
    #!/usr/bin/env bash
    set -euo pipefail
    export PATH="$HOME/.local/bin:$PATH"
    ggen sync run --dry-run
