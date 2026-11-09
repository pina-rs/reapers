#!/usr/bin/env bash
# Tools a cloud agent needs before `cargo xtask check`.
#
# Idempotent: a second run skips anything already installed at the pinned
# version. Versions match `.github/workflows/ci.yml`. The base image is
# Cursor's default Ubuntu image, which already has rustup, a C toolchain
# and passwordless sudo. iOS and Android app builds are not installed:
# they need Xcode or the Android SDK, and the checks do not build them.

set -euo pipefail

cd "$(dirname "$0")/.."

export DEBIAN_FRONTEND=noninteractive
export PATH="/usr/local/bin:${CARGO_HOME:-$HOME/.cargo}/bin:${PATH}"

AGAVE_VERSION=v4.1.0
PINA_VERSION=0.24.0
LEPTOSKIT_REV=44cfd118a6ab5aa6e98eb641048fa307c91e47d2
PG_BIN=/usr/lib/postgresql/17/bin
AGAVE_BIN="${HOME}/.local/share/solana/install/active_release/bin"
CARGO_BIN="${CARGO_HOME:-$HOME/.cargo}/bin"
LEPTOSKIT_STAMP="${HOME}/.cache/reapers/leptoskit-rev"

sudo apt-get update
sudo apt-get install -y --no-install-recommends \
	build-essential \
	ca-certificates \
	clang \
	cmake \
	curl \
	git \
	pkg-config \
	perl \
	python3

if [[ ! -x "${PG_BIN}/initdb" ]]; then
	sudo apt-get install -y --no-install-recommends postgresql-common
	sudo /usr/share/postgresql-common/pgdg/apt.postgresql.org.sh -y
	sudo apt-get install -y --no-install-recommends postgresql-17
fi

# The package starts a cluster. The server's tests start their own, and
# a system cluster would hold port 5432.
if command -v systemctl >/dev/null 2>&1; then
	sudo systemctl disable --now postgresql >/dev/null 2>&1 || true
fi

for tool in initdb pg_ctl createdb postgres pg_isready; do
	if [[ -x "${PG_BIN}/${tool}" ]]; then
		sudo ln -sfn "${PG_BIN}/${tool}" "/usr/local/bin/${tool}"
	fi
done

if ! command -v rustup >/dev/null 2>&1; then
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
	sudo ln -sfn "${CARGO_BIN}/rustup" /usr/local/bin/rustup
	sudo ln -sfn "${CARGO_BIN}/cargo" /usr/local/bin/cargo
	sudo ln -sfn "${CARGO_BIN}/rustc" /usr/local/bin/rustc
fi

# A self-update changes the toolchain installer under a running agent.
rustup set auto-self-update disable
rustup toolchain install
rustup target add \
	wasm32-unknown-unknown \
	wasm32-wasip1 \
	aarch64-linux-android \
	aarch64-apple-ios \
	aarch64-apple-ios-sim

if ! "${AGAVE_BIN}/solana" --version 2>/dev/null | grep -q '4\.1\.0'; then
	sh -c "$(curl --proto '=https' --tlsv1.2 -sSfL "https://release.anza.xyz/${AGAVE_VERSION}/install")"
fi

# cargo-build-sbf finds the platform tools beside itself. Link the whole
# release bin directory, resolving through the symlink, so that holds.
if [[ -d "${AGAVE_BIN}" ]]; then
	for bin in "${AGAVE_BIN}"/*; do
		[[ -f "${bin}" && -x "${bin}" ]] || continue
		sudo ln -sfn "${bin}" "/usr/local/bin/$(basename "${bin}")"
	done
fi

if ! command -v pina >/dev/null 2>&1 || ! pina --version 2>/dev/null | grep -q "${PINA_VERSION}"; then
	cargo install pina_cli --version "${PINA_VERSION}" --locked
fi

mkdir -p "$(dirname "${LEPTOSKIT_STAMP}")"
if ! command -v leptoskit >/dev/null 2>&1 || [[ "$(cat "${LEPTOSKIT_STAMP}" 2>/dev/null || true)" != "${LEPTOSKIT_REV}" ]]; then
	cargo install leptoskit_cli --locked \
		--git https://github.com/leptoskit/leptoskit \
		--rev "${LEPTOSKIT_REV}"
	printf '%s\n' "${LEPTOSKIT_REV}" >"${LEPTOSKIT_STAMP}"
fi

for name in pina leptoskit; do
	if [[ -x "${CARGO_BIN}/${name}" ]]; then
		sudo ln -sfn "${CARGO_BIN}/${name}" "/usr/local/bin/${name}"
	fi
done

cargo fetch --locked

echo "cloud-install: ready"
rustc --version
initdb --version
solana --version
cargo-build-sbf --version
pina --version
leptoskit --version
