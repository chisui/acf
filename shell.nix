{ sources ? import ./nix/sources.nix }:
let pkgs = import sources.nixpkgs {
  overlays = [
    (import sources.nixpkgs-mozilla)
  ];
};
in with pkgs;
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
