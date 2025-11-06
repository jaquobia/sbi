{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };
  outputs =
    inputs@{ self, nixpkgs }:
    let
      forEachSystem =
        function:
        nixpkgs.lib.genAttrs nixpkgs.lib.systems.flakeExposed (
          system: function nixpkgs.legacyPackages.${system}
        );
    in
    {
      # packages = forEachSystem(pkgs: {
      # default = pkgs.callPackage ./. {};
      # });
      devShells = forEachSystem (
        pkgs:
        let
          libPath =
            with pkgs;
            pkgs.lib.makeLibraryPath [
              wayland
              libxkbcommon
              vulkan-headers
              vulkan-loader
              libGL
              libGLU
              SDL2
            ];
        in
        {
          default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              cargo
              clippy
              rustc
              rustfmt
              rust-analyzer
              pkg-config
            ];
            buildInputs = with pkgs; [
              wayland
            ];
            env = {
              RUST_BACKTRACE = "1"; # full
              RUSTFLAGS = "-C link-arg=-Wl,-rpath,${libPath}";
              LD_LIBRARY_PATH = libPath;
            };
          };
        }
      );
    };
}
