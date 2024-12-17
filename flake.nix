{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let pkgs = import nixpkgs { inherit system; };

      in {
        packages = rec {
          typst-web-preview = pkgs.rustPlatform.buildRustPackage {
            name = "typst-web-preview";
            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
              allowBuiltinFetchGit = true;
            };

            preBuild = ''
              mkdir -p web/dist
              cp -r ${typst-web-preview-static}/* web/dist
            '';
          };

          typst-web-preview-static = pkgs.buildNpmPackage {
            name = "typst-web-preview-static";
            src = ./web;

            npmDepsHash = "sha256-cRVFDlyG30N3/YR6idHdO1t4UgO4vUfTYCdC2dLpm54=";

            installPhase = ''
              mkdir $out
              cp -r dist/* $out
            '';
          };

          default = typst-web-preview;
        };
      });
}
