{ pkgs, rustPlatform }:
let
  inherit (pkgs) lib;
in
rustPlatform.buildRustPackage {
  pname = "dirtabase";
  version = "0.1";
  cargoLock.lockFile = ./Cargo.lock;
  src = lib.cleanSource ./.;

  buildInputs =
    with pkgs;
    [ openssl ]
    ++ lib.optionals pkgs.stdenv.isDarwin [
      libiconv
      darwin.apple_sdk.frameworks.SystemConfiguration
    ];

  nativeBuildInputs = with pkgs; [ pkg-config ];
}
