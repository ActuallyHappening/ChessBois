set -exuo pipefail

./macos-release.sh &
./windows-release.sh &
./web-release &
