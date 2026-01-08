{
  description = "ffcv - Firefox Configuration Viewer";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        packages.ffcv = pkgs.rustPlatform.buildRustPackage {
          pname = "ffcv";
          version = "1.0.2";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          buildType = "release";
        };

        defaultPackage = self.packages.${system}.ffcv;

        devShells.default = pkgs.mkShell {
          buildInputs = [ pkgs.rustc pkgs.cargo ];
        };
      });
}
