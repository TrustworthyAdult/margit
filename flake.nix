{
  description = "The prettiest git CLI.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { self, nixpkgs, rust-overlay }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems =
        f:
        nixpkgs.lib.genAttrs systems (
          system:
          f (import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          })
        );
    in
    {
      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          packages = [
            # Single source of truth: channel + components come from
            # ./rust-toolchain.toml, which rustup-based contributors honor too.
            (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)

            # rustc shells out to `cc` to link the final binary. NixOS keeps no
            # compiler on the default PATH, so without this `cargo build`/`test`
            # fails with `linker `cc` not found`.
            pkgs.stdenv.cc
          ];
        };
      });
    };
}
