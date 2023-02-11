{ pkgs ? import <nixpkgs> { } }:
let
  dfx-env = import (fetchTarball https://github.com/ninegua/ic-nix/releases/latest/download/dfx-env.tar.gz) { version = "20230101"; };
in
dfx-env.overrideAttrs (old: {
  nativeBuildInputs = old.nativeBuildInputs ++ [ pkgs.rustup pkgs.pkg-config pkgs.openssl pkgs.protobuf pkgs.cmake pkgs.cachix pkgs.killall ];
})
