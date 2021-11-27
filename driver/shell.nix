{ pkgs ? import <nixpkgs> { } }:

with pkgs;

mkShell {
  nativeBuildInputs = [ pkg-config ];
  # i also have openssl so cargo-edit works
  buildInputs = [ openssl ffmpeg-full ];
}
