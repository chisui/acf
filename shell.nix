{ pkgs ? import <nixpkgs> {} }:
with pkgs;
mkShell {
  name = "acf";
  buildInputs = [
    rustc
    cargo
    rls
  ];
}
