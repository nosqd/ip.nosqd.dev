{
  description = "nosqd's IP info service";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      crane,
      ...
    }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ (import rust-overlay) ];
      };

      rustToolchain = pkgs.rust-bin.stable.latest.default;
      craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

      mmdb-city = pkgs.fetchurl {
        url = "https://github.com/P3TERX/GeoLite.mmdb/releases/download/2026.03.13/GeoLite2-City.mmdb";
        sha256 = "sha256-z9TMqb3aiJbkcL9gKVA8BBQoZ9TwVe4jA5QXVZDhMUU=";
      };
      mmdb-asn = pkgs.fetchurl {
        url = "https://github.com/P3TERX/GeoLite.mmdb/releases/download/2026.03.13/GeoLite2-ASN.mmdb";
        sha256 = "sha256-Llmk5CZzi4URr+X4gcXHD0mcMatUgRipigycl7/Od0U=";
      };

      ip-nosqd = craneLib.buildPackage {
        src = craneLib.cleanCargoSource (craneLib.path ./.);
        buildInputs = [ ];
        nativeBuildInputs = [ ];
      };
    in
    {
      packages.${system} = {
        default = ip-nosqd;
        docker = pkgs.dockerTools.buildImage {
          name = "ip-nosqd";
          tag = "latest";
          copyToRoot = pkgs.buildEnv {
            name = "image-root";
            paths = [ pkgs.cacert ];
          };
          config = {
            Cmd = [ "${ip-nosqd}/bin/ip-nosqd-dev" ];
            Env = [
              "CITY_DB_PATH=${mmdb-city}"
              "ASN_DB_PATH=${mmdb-asn}"
              "PORT=3000"
            ];
            ExposedPorts = {
              "3000/tcp" = { };
            };
          };
        };
      };

      devShells.${system}.default = pkgs.mkShell {
        packages = [
          rustToolchain
          pkgs.rust-analyzer
        ];
      };
    };
}
