set -exu

./macos-release.sh &
./windows-release.sh &
./web-release &

# scp -r "ah@ahubuntu:/home/ah/ChessBois/releases/Caleb's MSRCQ11 v0.1.9" .
