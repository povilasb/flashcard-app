{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    nodejs
    duckdb
    openssl
    pkg-config
  ];

  nativeBuildInputs = [
    pkgs.clang
    pkgs.libcxxStdenv
  ];
}