{ pkgs, rustPlatform }:
let
  inherit (pkgs) lib;
in
rustPlatform.buildRustPackage rec {
  pname = "dirtabase";
  version = "0.1";
  cargoLock.lockFile = ./Cargo.lock;
  src = lib.cleanSource ./.;

  doCheck = false;

  buildInputs = with pkgs; [ openssl ];

  nativeBuildInputs = with pkgs; [ pkg-config ];
}
