{}:
let
  rev = "53dad94e874c9586e71decf82d972dfb640ef044";
  nixpkgs = fetchTarball "https://github.com/NixOS/nixpkgs/archive/${rev}.tar.gz";
  pkgs = import nixpkgs { };
  dfx-env = import (builtins.fetchTarball "https://github.com/ninegua/ic-nix/releases/latest/download/dfx-env.tar.gz") { version = "20230330"; inherit pkgs; };
in
dfx-env.overrideAttrs (old: {
  nativeBuildInputs = with pkgs; old.nativeBuildInputs ++
    [ rustup pkg-config openssl protobuf cmake cachix killall ];

  shellHook = ''
    rustup toolchain install stable
    rustup target add wasm32-unknown-unknown
  '';
})

# { pkgs ? import <nixpkgs> { } }:
# let
#   dfx-env = import (builtins.fetchTarball "https://github.com/ninegua/ic-nix/releases/latest/download/dfx-env.tar.gz") { version = "20230330"; };
# in
# dfx-env.overrideAttrs (old: {
#   nativeBuildInputs = with pkgs; old.nativeBuildInputs ++
#     [ rustup pkg-config openssl protobuf cmake cachix killall ];

#   shellHook = ''
#     rustup toolchain install stable
#     rustup target add wasm32-unknown-unknown
#   '';
# })
