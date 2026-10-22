{
  description = "New development project. TODO: change.";

  inputs = {
    # Nixpkgs
    nixpkgs = {
      url = "https://channels.nixos.org/nixpkgs-unstable/nixexprs.tar.zst";
    };
    # Naersk
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # Use pre-commit-hooks to run linters/checks etc.
    pre-commit-hooks = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-compat.follows = "";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      naersk,
      pre-commit-hooks,
      ...
    }@inputs:
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

      packages = builtins.mapAttrs (
        system: _pkgs:
        let
          pkgs = self.overlays.default _pkgs _pkgs;
        in
        {
          inherit (pkgs) nbd-rs;
          default = pkgs.nbd-rs;
        }
      ) nixpkgs.legacyPackages;

      checks = builtins.mapAttrs (system: pkgs: {
        pre-commit-check = pre-commit-hooks.lib.${system}.run {
          src = ./.;
          settings = {
            rust.check.cargoDeps = pkgs.rustPlatform.importCargoLock {
              lockFile = ./Cargo.lock;
            };
          };
          hooks = {
            # Format nix files
            nixfmt.enable = true;
            # Don't accidentally commit merge conflits
            check-merge-conflicts.enable = true;
            # Fix line-endings and eofs
            end-of-file-fixer.enable = true;
            mixed-line-endings.enable = true;
            # rustfmt / Clippy lints
            rustfmt.enable = true;
            clippy.enable = true;
          };
        };
      }) nixpkgs.legacyPackages;

      devShells = builtins.mapAttrs (system: pkgs: {
        default =
          let
            pre-commit = self.checks.${system}.pre-commit-check;
          in
          pkgs.mkShell {
            buildInputs = pre-commit.enabledPackages;

            packages = with pkgs; [
              cargo
              rustc
              rust-analyzer
              clippy
              rustfmt
              nbd
            ];
            RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;

            shellHook = ''
              ${pre-commit.shellHook}
              export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
            '';
          };
      }) nixpkgs.legacyPackages;
    };
}
