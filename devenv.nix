{ pkgs, ... }:
{
  enterShell = ''
    Hello, $USER. Welcome to the dirtabase development shell.
  '';

  packages = with pkgs; [ dirtabase ];

  # formats
  pre-commit.hooks.check-json.enable = true;
  pre-commit.hooks.check-toml.enable = true;
  pre-commit.hooks.check-yaml.enable = true;
  pre-commit.hooks.trim-trailing-whitespace.enable = true;

  # scm
  pre-commit.hooks.check-merge-conflicts.enable = true;
  pre-commit.hooks.check-added-large-files.enable = true; # prohibit "very large" files
  pre-commit.hooks.check-case-conflicts.enable = true; # macos fs is case-insensitive; helps linux

  # nix
  pre-commit.hooks.deadnix.enable = true; # dead nix code remover
  pre-commit.hooks.nixfmt.enable = true;
  pre-commit.hooks.nixfmt.package = pkgs.nixfmt-rfc-style;

  # rust
  languages.rust.enable = true;
  pre-commit.hooks.rustfmt.enable = true;
  # pre-commit.hooks.rust-analyzer.enable = true;

  # shell
  pre-commit.hooks.bats.enable = true; # run bash unit tests
  pre-commit.hooks.beautysh.enable = true; # sh autoformatter
  pre-commit.hooks.shellcheck.enable = true; # sh linter
  pre-commit.hooks.check-shebang-scripts-are-executable.enable = true;
}
