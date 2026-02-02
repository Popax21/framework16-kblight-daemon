{
  description = "Daemon to manage / synchronize the Framework 16's keyboard backlight";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
  };

  outputs = {
    nixpkgs,
    flake-utils,
    crane,
    ...
  }:
    (flake-utils.lib.eachSystem (builtins.filter (nixpkgs.lib.hasInfix "linux") flake-utils.lib.defaultSystems) (system: let
      pkgs = nixpkgs.legacyPackages.${system};
      craneLib = crane.mkLib pkgs;
      flakePkg = pkgs.callPackage nix/package.nix {inherit craneLib;};
    in rec {
      packages.default = flakePkg;

      checks = {
        build = flakePkg;
        rustfmt = craneLib.cargoFmt {
          src = ./.;
        };
        clippy = craneLib.cargoClippy {
          src = ./.;
          inherit (flakePkg) cargoArtifacts buildInputs nativeBuildInputs;
          cargoClippyExtraArgs = "--all-targets --all-features -- --deny warnings";
        };
      };

      devShells.default = craneLib.devShell {
        inherit checks;
        propagatedBuildInputs = [flakePkg.cargoArtifacts]; # - keep our cargo artifacts alive as part of the direnv GC root
        packages = with pkgs; [
          clippy
          rust-analyzer
        ];
      };
    }))
    // rec {
      overlays.default = final: prev: {
        fw16-kblight-daemon = final.callPackage nix/package.nix {craneLib = crane.mkLib final;};
      };
      nixosModules.default = {
        imports = [./nix/nixos/module.nix];
        nixpkgs.overlays = [overlays.default];
      };
    };
}
