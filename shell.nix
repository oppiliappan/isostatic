{
  nixpkgs ? ./nix/nixpkgs.nix
  , pkgs ? import nixpkgs
}:

pkgs.mkShell {
  buildInputs = [
    pkgs.cargo
    pkgs.rustc
    pkgs.git
    pkgs.shellcheck
    pkgs.sqlite
    pkgs.httpie
  ];
}
