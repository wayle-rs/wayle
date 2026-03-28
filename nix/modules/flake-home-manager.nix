# This is just a wrapper for the main wayle home manager module that is only
# called by the flake.
#
# It simply overrides the default package for the service to the one
# defined/used by the flake, which itself constructs the package using rust
# nightly. This probably won't be the case forever which is why the actual
# modules has been made separate for (when the time comes) easier porting to
# wayle being a regular nix home manager package.
self: {
  config,
  lib,
  pkgs,
  ...
} @ s:
import ./home-manager.nix (
  s
  // {
    pkgs =
      pkgs
      // {
        wayle = self.packages.${pkgs.stdenv.system}.wayle;
      };
  }
)
