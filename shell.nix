{ nixpkgs ? import <nixpkgs> {} }:

with nixpkgs;

stdenv.mkDerivation {
  name = "buildenv";

  buildInputs = [
    ncurses.dev
  ];

  shellHook = ''
    NIX_ENFORCE_PURITY=0
    exec ${zsh}/bin/zsh
  '';
}
