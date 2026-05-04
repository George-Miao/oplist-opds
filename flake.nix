{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    crane = {
      url = "github:ipetkov/crane";
    };

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      fenix,
    }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};

      toolchain = fenix.packages.${system}.stable.toolchain;
      craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
      commonArgs = {
        src = craneLib.cleanCargoSource ./.;
        strictDeps = true;
      };

      cargoArtifacts = craneLib.buildDepsOnly commonArgs;

      oplist-opds = craneLib.buildPackage (
        commonArgs
        // {
          inherit cargoArtifacts;
        }
      );

      oplist-opds-docker = pkgs.dockerTools.buildImage {
        name = "oplist-opds";
        tag = "latest";

        config = {
          Cmd = [ "/bin/oplist-opds" ];
          ExposedPorts = {
            "3000/tcp" = { };
          };
          Env = [ "RUST_LOG=info" ];
        };

        copyToRoot = pkgs.buildEnv {
          name = "image-root";
          paths = [
            oplist-opds
            pkgs.cacert # CA certs for TLS connections to OpenList
          ];
          pathsToLink = [
            "/bin"
            "/etc"
          ];
        };
      };
    in
    {
      packages.${system} = {
        inherit oplist-opds oplist-opds-docker;
        default = oplist-opds-docker;
      };

      devShells.${system}.default = pkgs.mkShell {
        buildInputs = [ toolchain ];
      };
    };
}
