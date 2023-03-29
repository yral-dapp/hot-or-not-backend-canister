# TODO: update to the latest ic-nix release
{ pkgs ? import <nixos-22.11/nixpkgs> { } }:
let
  ic-nix = pkgs.fetchFromGitHub {
    owner = "ninegua";
    repo = "ic-nix";
    rev = "0c8c6808a88d650b5546a60a073853f6ceae97cc";
    sha256 = "sha256-MiK4vw4TLjXN3VH24+yeUvV7Qy2zbtKr5io/lafXjVA=";
  };
  dfx-env = pkgs.callPackage "${ic-nix}/dfx-env.nix" {
    force = true;
    inherit pkgs ic-nix;
  };
in
# modify to include installing rust stable and adding wasm32 target
dfx-env.overrideAttrs (old: {
  nativeBuildInputs = old.nativeBuildInputs ++ [ pkgs.rustup pkgs.pkg-config pkgs.openssl pkgs.protobuf pkgs.cmake pkgs.cachix pkgs.killall ];
})
