{
  description = "Flake for hot-or-not-backend-canister";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/1c3a28d84f970e7774af04372ade06399add182e";
    ic-nix = {
      url = "github:ninegua/ic-nix/20240610";
      flake = false;
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      ic-nix,
      ...
    }:
    let
      pkgs = nixpkgs.legacyPackages.x86_64-linux;
      dfx-env = pkgs.callPackage "${ic-nix}/dfx-env.nix" {
        force = true;
      };
    in
    {
      devShells.x86_64-linux.default = pkgs.mkShell {
        buildInputs = [
          dfx-env
          pkgs.rustup
          pkgs.pkg-config
          pkgs.openssl
          pkgs.protoc
          pkgs.cmake
          pkgs.cachix
          pkgs.killall
          pkgs.jq
          pkgs.coreutils
          pkgs.bc
          pkgs.python3Full
          pkgs.libiconv
          pkgs.wget
        ] ++ (if pkgs.stdenv.isDarwin then [ pkgs.darwin.libiconv ] else [ ]);
      };
    };
}
