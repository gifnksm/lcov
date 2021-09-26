#!/bin/bash -eu

cd "$(dirname $(readlink -f $0))"

cargo build --workspace
cargo test --workspace
cargo doc --workspace

cargo package

PKG_VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml)"
PKG_NAME="$(sed -n 's/^name = "\(.*\)"/\1/p' Cargo.toml)"

echo "PKG: ${PKG_NAME}_v${PKG_VERSION}"

read -n1 -p "ok? (y/N): " yn
echo
case "$yn" in
    [yY]*) ;;
    *)
        echo "cancelled" >&2
        exit 1
        ;;
esac

cargo publish
GIT_COMMITTER_DATE=$(git log -n1 --pretty=%aD) git tag -a -m "Release ${PKG_NAME} v${PKG_VERSION}" "${PKG_NAME}_v${PKG_VERSION}"
git push --tags
