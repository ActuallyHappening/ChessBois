set -exu

./macos-release.sh &
./windows-release.sh &
./web-release &

# scp -r "ah@ahubuntu:/home/ah/ChessBois/releases/*.dist" .
