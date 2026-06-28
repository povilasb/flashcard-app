{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    nodejs
    sqlite
    openssl
    pkg-config
  ];

  nativeBuildInputs = [
    pkgs.clang
  ];
}
