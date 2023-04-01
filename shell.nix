{ pkgs ? import <nixpkgs> { } }:
let
  dfx-env = import (fetchTarball https://github.com/ninegua/ic-nix/releases/latest/download/dfx-env.tar.gz) { version = "20230330"; };
in
dfx-env.overrideAttrs (old: {
  nativeBuildInputs = with pkgs; old.nativeBuildInputs ++
    [ rustup pkg-config openssl protobuf cmake cachix killall ];

  shellHook = ''
    rustup toolchain install stable
    rustup target add wasm32-unknown-unknown
  '';
})
