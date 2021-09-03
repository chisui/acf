{ pkgs ? import <nixpkgs> {
    overlays = [
       (import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz))
    ];
  }
}:
with pkgs;
let
  rust-channel = rustChannelOf {
    date = "2021-08-15";
    channel = "nightly";
  };
in mkShell {
  name = "acf";
  buildInputs = [
    (rust-channel.rust.override {
      extensions = [
        "rust-src"
        "rls-preview"
      ];
    })
  ];
}
