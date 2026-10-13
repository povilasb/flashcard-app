{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [ pkgs.openssl pkgs.sqlite ];

  buildInputs = with pkgs; [
    bun
    sqlite
    openssl
    pkg-config
  ];

  nativeBuildInputs = [
    pkgs.clang
  ];
}
