{}:
let
  rev = "1c3a28d84f970e7774af04372ade06399add182e";
  nixpkgs = fetchTarball "https://github.com/NixOS/nixpkgs/archive/${rev}.tar.gz";
  pkgs = import nixpkgs { };
  dfx-env = import (builtins.fetchTarball "https://github.com/ninegua/ic-nix/releases/download/20240610/dfx-env.tar.gz") { version = "20240610"; inherit pkgs; };
in
dfx-env.overrideAttrs (old: {
  nativeBuildInputs = with pkgs; old.nativeBuildInputs ++
    [
      rustup pkg-config openssl protobuf cmake cachix killall jq coreutils bc python3Full
    ] ++ (if pkgs.stdenv.isDarwin then [
      darwin.apple_sdk.frameworks.Foundation
      pkgs.darwin.libiconv
    ] else []);
    shellHook = ''
      cargo install --root $out --force candid-extractor
      ln -s $out/bin/candid-extractor $out/bin/candid-extractor
      export PATH="$out/bin:$PATH"
    '';
})
