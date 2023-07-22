{}:
let
  rev = "cc45a3f8c98e1c33ca996e3504adefbf660a72d1";
  nixpkgs = fetchTarball "https://github.com/NixOS/nixpkgs/archive/${rev}.tar.gz";
  pkgs = import nixpkgs { };
  dfx-env = import (builtins.fetchTarball "https://github.com/ninegua/ic-nix/releases/download/20230704/dfx-env.tar.gz") { version = "20230704"; inherit pkgs; };
in
dfx-env.overrideAttrs (old: {
  nativeBuildInputs = with pkgs; old.nativeBuildInputs ++
    [ rustup pkg-config openssl protobuf cmake cachix killall jq coreutils bc python3Full ];

  # shellHook = ''
  #   rustup toolchain install stable
  #   rustup target add wasm32-unknown-unknown
  # '';
})
