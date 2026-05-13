# APP1

To compile and deploy the code, you need two ESP32.

The simplest way to flash is by using a linux machine with nix installed:
Start by opening a nix shell using `nix develop`.
Then simply run `cargo run --release -p hub` or `cargo run --release -p sensor` depending on your target.
