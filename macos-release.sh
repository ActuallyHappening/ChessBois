set -e

# fail, please confirm cargo config

cargo +nightly build --release --target x86_64-apple-darwin

# Set ENV_BINARY to bevy_solver
ENV_BINARY=bevy_solver

mkdir -p $ENV_BINARY.app/Contents/MacOS
cp target/x86_64-apple-darwin/release/$ENV_BINARY $ENV_BINARY.app/Contents/MacOS/
cp -r assets $ENV_BINARY.app/Contents/MacOS/
hdiutil create -fs HFS+ -volname "$ENV_BINARY" -srcfolder $ENV_BINARY.app $ENV_BINARY.dmg

rm -rf $ENV_BINARY.app

# Command to find version: cargo get version --pretty
# New name: "Caleb's MSRCQ11 {}.dmg" replacing {} with version

VERSION=$(cargo get version --pretty)
mv $ENV_BINARY.dmg "releases/Caleb's MSRCQ11 ${VERSION}.dmg"
