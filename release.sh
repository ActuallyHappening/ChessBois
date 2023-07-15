set -exuo pipefail

./macos-release.sh &
./windows-release.sh &
trunk build --release &
