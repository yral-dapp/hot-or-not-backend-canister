{}:
let
  rev = "cc45a3f8c98e1c33ca996e3504adefbf660a72d1";
  nixpkgs = fetchTarball "https://github.com/NixOS/nixpkgs/archive/${rev}.tar.gz";
  pkgs = import nixpkgs { };
  dfx-env = import (builtins.fetchTarball "https://github.com/ninegua/ic-nix/releases/latest/download/dfx-env.tar.gz") { version = "20230508"; inherit pkgs; };
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
