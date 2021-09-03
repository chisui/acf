{ pkgs ? import <nixpkgs> {
    overlays = [
       (import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz))
    ];
  }
}:
pkgs.mkShell {
  name = "acf";
  buildInputs = [
    pkgs.latest.rustChannels.nightly.rust
    pkgs.rls
  ];
}
