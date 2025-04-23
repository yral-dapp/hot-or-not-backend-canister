{
  description = "Hot-or-not backend canister development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/1c3a28d84f970e7774af04372ade06399add182e";
    flake-utils.url = "github:numtide/flake-utils";
    ic-nix = {
      url = "github:ninegua/ic-nix";
      flake = false;
    };
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      ic-nix,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };

        # Use ic-nix input directly rather than fetchTarball
        dfx-env = import "${ic-nix}/dfx-env.nix" {
          inherit pkgs;
          force = true;
        };
      in
      {
        devShells.default = dfx-env.overrideAttrs (old: {
          nativeBuildInputs =
            with pkgs;
            old.nativeBuildInputs
            ++ [
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
                  darwin.libiconv
                ]
              else
                [ ]
            );

          shellHook = ''
            # Ensure the DFX environment is properly set up
            export DFX_CONFIG_ROOT="$PWD/.dfx/config"
            export DFX_CACHE_ROOT="/tmp/dfx-cache"

            echo "Installing candid-extractor..."
            if ! command -v candid-extractor &> /dev/null; then
              cargo install --quiet candid-extractor
            fi
          '';
        });
      }
    );
}
