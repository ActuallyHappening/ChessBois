set -exu

TARGET_NAME=x86_64-pc-windows-gnu
ENV_BINARY=$(cargo get --name)

cargo bwindows --release --no-default-features

mkdir -p $ENV_BINARY.zipfolder
cp target/$TARGET_NAME/release/$ENV_BINARY.exe $ENV_BINARY.zipfolder/
# Cp assets
cp -r assets $ENV_BINARY.zipfolder/
# Zip
zip -r $ENV_BINARY.zip $ENV_BINARY.zipfolder

rm -rf $ENV_BINARY.zipfolder

VERSION=$(cargo get version --pretty)
mv $ENV_BINARY.zip "releases/Caleb's MSRCQ11 ${VERSION}.zip"
