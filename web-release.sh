set -exu

CARGO_PROFILE_RELEASE_OPT_LEVEL="s" trunk build --release --no-default-features

VERSION=$(cargo get version --pretty)
cp -r dist "releases/Caleb's MSRCQ11 ${VERSION}"
