{ pkgs ? import <nixpkgs> { } }:
let
  dfx-env = import (fetchTarball https://github.com/ninegua/ic-nix/releases/latest/download/dfx-env.tar.gz) { version = "20230330"; };
in
# TODO: modify to include installing rust stable and adding wasm32 target
dfx-env.overrideAttrs (old: {
  nativeBuildInputs = with pkgs; old.nativeBuildInputs ++
    [ rustup pkg-config openssl protobuf cmake cachix killall ];
})
