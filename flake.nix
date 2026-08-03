{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs =
    {
      self,
      nixpkgs,
      naersk,
      ...
    }:
    let
      supportedSystems = [
        "aarch64-linux"
        "x86_64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
      ];
      forEachSupportedSystem =
        f:
        nixpkgs.lib.genAttrs supportedSystems (
          system:
          f {
            pkgs = import nixpkgs {
              inherit system;
              overlays = [ self.overlays.default ];
            };
          }
        );
    in
    {
      overlays.default = final: prev: {
        nbd-rs =
          let
            naersk' = prev.callPackage naersk { };
          in
          naersk'.buildPackage {
            src = ./.;
          };
      };
      devShells = forEachSupportedSystem (
        { pkgs }:
        {
          default = pkgs.mkShell {
            strictDeps = true;
            nativeBuildInputs = with pkgs; [
              cargo
              rustc
              rust-analyzer
              clippy
              rustfmt
              nbd
            ];
            RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
          };
        }
      );
      packages = forEachSupportedSystem (
        { pkgs }:
        rec {
          nbd-rs = pkgs.nbd-rs;
          default = nbd-rs;
        }
      );
    };
}
