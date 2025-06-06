{ pkgs ? import <nixpkgs> { } }:
{
	sbi = pkgs.callPackage ./derivative.nix {};
}
