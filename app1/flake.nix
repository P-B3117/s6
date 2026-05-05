{
  description = "Wrapper for esp-rs-nix devShell";

  inputs = {
    esp-rs-nix.url = "github:leighleighleigh/esp-rs-nix";
  };

  outputs = { self, esp-rs-nix, ... }:
  let
    system = "x86_64-linux"; # change if needed
  in {
    devShells.${system}.default =
      esp-rs-nix.devShells.${system}.default;
  };
}
