set -exu

trunk build --release

VERSION=$(cargo get version --pretty)
cp -r dist "releases/Caleb's MSRCQ11 ${VERSION}.dist"
