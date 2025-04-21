{
  description = "Flake for hot-or-not-backend-canister";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/1c3a28d84f970e7774af04372ade06399add182e";
    dfx-env = {
      url = "github:ninegua/ic-nix";
      flake = false;
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      dfx-env,
    }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
    in
    {
      devShell.${system} = pkgs.mkShell {
        nativeBuildInputs =
          with pkgs;
          [
            rustup
            pkg-config
            openssl
            protobuf
            cmake
            cachix
            killall
            jq
            coreutils
            bc
            python3Full
            libiconv
            wget
          ]
          ++ (
            if pkgs.stdenv.isDarwin then
              [
                darwin.apple_sdk.frameworks.Foundation
                pkgs.darwin.libiconv
              ]
            else
              [ ]
          );
        shellHook = ''
          cargo install --root $out --force candid-extractor
          ln -s $out/bin/candid-extractor $out/bin/candid-extractor
          export PATH="$out/bin:$PATH"
        '';
      };
    };
}
