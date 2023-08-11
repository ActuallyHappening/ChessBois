set -exu



CARGO_PROFILE_RELEASE_OPT_LEVEL="s" trunk build --release --no-default-features

VERSION=$(cargo get version --pretty)
readonly NAME="Caleb's MSRCQ11 ${VERSION}"
cp -r dist "releases/${NAME}"

rsync -r "./releases/${NAME}" "digitalocean:/root/ChessBois/dist" 