{
  description = "A fast, configurable desktop environment shell for Wayland compositors.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    self.submodules = true;
  };

  outputs = {
    self,
    nixpkgs,
    ...
  } @ inputs: let
    inherit (nixpkgs) lib;

    supportedSystems = [
      "x86_64-linux" # 64-bit Intel/AMD Linux
      "aarch64-linux" # 64-bit ARM Linux
    ];
    forAllSystems = lib.genAttrs supportedSystems;
  in {
    packages = forAllSystems (system: let
      pkgs = nixpkgs.legacyPackages.${system};
      toolchain = inputs.fenix.packages.${system}.minimal.toolchain;

      rustPlatform = pkgs.makeRustPlatform {
        cargo = toolchain;
        rustc = toolchain;
      };
    in {
      default = self.packages.${system}.wayle;
      wayle =
        (pkgs.callPackage ./nix/package.nix {
          inherit rustPlatform;
        }).overrideAttrs (
          oldAttrs: {
            src = self;
          }
        );
    });

    homeManagerModules = {
      default = self.homeManagerModules.wayle;
      wayle = import ./nix/modules/flake-home-manager.nix self;
    };
  };
}
