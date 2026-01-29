{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };

      toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

      kleinos = pkgs.stdenv.mkDerivation {
        pname = "kleinos";
        version = "0.1.0";

        src = ./.;

        nativeBuildInputs = [
          toolchain
          pkgs.pkg-config
        ];

        # Required for build-std
        RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";

        buildPhase = ''
          export HOME=$(mktemp -d)
          cargo build --release
        '';

        installPhase = ''
          mkdir -p $out/bin
          cp target/x86_64-kleinos/release/kleinos $out/bin/
        '';
      };
    in {
      packages.${system}.default = kleinos;

      devShells.${system}.default = pkgs.mkShell {
        buildInputs = [
          toolchain
          pkgs.pkg-config
          pkgs.cargo-bootimage
          pkgs.qemu
          pkgs.gdb
        ];
        RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
      };
    };
}
