# CAP: Chess Analysis Program
View an up-to-date online version here: https://caleb-msrc-q11.netlify.app.
Download a desktop version in this repo's releases page: https://github.com/ActuallyHappening/ChessBois/releases

## What is it?
CAP	is a program to visually analyse chess boards in the context of knights tours.
It can quickly and easily construct board topologies which you can then search for knight's tours on,
by simply hovering your mouse on a start location.

**It is 100% Rust!**

You can download a desktop version, which also supports uploading and downloading board solutions.
If you are on Windows, download the `.zip` file. If you are on macOS, download the `.dmg` file.
You will need to disable security to run them, as they are not signed.

## To run yourself from source
First install rust.
Prerequisites: `brew install llvm && cargo install cargo-watch && cargo install trunk` (macos/linux only).

Clone local dep (at root of project): `git clone https://github.com/ActuallyHappening/bevy_egui_controls.git`

Then, `cargo r` / `cargo w` to run and reload the project respectively.
`./web-dev.sh` to view in a web browser.

<!-- // echo 'export PATH="/opt/homebrew/opt/llvm/bin:$PATH"' >> ~/.zshrc -->
<!-- $env.path = $env.path | prepend '/opt/homebrew/opt/llvm/bin' -->
