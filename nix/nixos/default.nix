#Non-flake NixOS module entrypoint
{
  imports = [./module.nix];
  nixpkgs.overlays = [(import ../overlay.nix)];
}
