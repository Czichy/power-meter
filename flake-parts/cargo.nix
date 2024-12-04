{inputs, ...}: {
  perSystem = {
    config,
    # pkgs,
    system,
    inputs',
    self',
    lib,
    ...
  }:
    with inputs; let
      manifest = (pkgs.lib.importTOML ../Cargo.toml).package;
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          (import rust-overlay)
          (final: prev: {
            lpi = inputs.lpi.packages.${system}.default;
          })
        ];
      };
      fenix-channel = fenix.packages.${system}.latest;

      fenix-toolchain = fenix-channel.withComponents [
        "rustc"
        "cargo"
        "clippy"
        "rust-src"
        "llvm-tools-preview"
      ];

      craneLib = (crane.mkLib pkgs).overrideToolchain fenix-toolchain;

      src = nix-filter.lib {
        root = ../.;
        include = [
          "Cargo.toml"
          "Cargo.lock"
          "taplo.toml"
          "rustfmt.toml"
          "rust-toolchain.toml"
          "src"
        ];
      };

      inherit (craneLib.crateNameFromCargoToml {inherit src;}) pname version;

      args = {
        inherit src;
        strictDeps = true;
        nativeBuildInputs = with pkgs; [
          openssl
          pkg-config
          udev
        ];
        buildInputs = with pkgs; [
          openssl.dev
          openssl
          pkg-config
          udev
        ];
        LD_LIBRARY_PATH = lib.makeLibraryPath [pkgs.openssl];
        # Needed to get openssl-sys to use pkg-config.
        # Doesn't seem to like OpenSSL 3
        OPENSSL_NO_VENDOR = 1;
        PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
        LIBCLANG_PATH = "${pkgs.libclang.lib}/lib/";
      };

      individualCrateArgs =
        args
        // {
          inherit cargoArtifacts version;
          doCheck = false;
        };

      fileSetForCrate = crateFiles:
        nix-filter.lib {
          root = ../.;
          include =
            [
              "src"
              "Cargo.toml"
              "Cargo.lock"
            ]
            ++ crateFiles;
        };

      cargoArtifacts = craneLib.buildDepsOnly args;

      power-meter = craneLib.buildPackage (individualCrateArgs
        // {
          pname = manifest.name;
          version = manifest.version;
          # cargoLock.lockFile = ./Cargo.lock;
          src =
            fileSetForCrate [
            ];
        });
    in {
      checks = {
        inherit power-meter;
        inherit (self.packages.${system}) services;

        clippy = craneLib.cargoClippy (args
          // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

        doc = craneLib.cargoDoc (args
          // {
            inherit cargoArtifacts;
          });

        fmt = craneLib.cargoFmt {
          inherit src;
        };

        toml-fmt = craneLib.taploFmt {
          src = pkgs.lib.sources.sourceFilesBySuffices src [".toml"];
          taploExtraArgs = "--config ../taplo.toml";
        };

        audit = craneLib.cargoAudit {
          inherit src advisory-db;
        };

        deny = craneLib.cargoDeny {
          inherit src;
        };

        nextest = craneLib.cargoNextest (args
          // {
            inherit cargoArtifacts;
            partitions = 1;
            partitionType = "count";
          });
      };

      packages = {
        inherit power-meter;
        inherit (self.checks.${system}) coverage;
        default = self.packages.${system}.power-meter;
      };
      legacyPackages = {
        cargoExtraPackages = args.nativeBuildInputs;
      };

      apps = {
        default = {
          program = self.packages.${system}.power-meter;
        };
      };

      formatter = pkgs.alejandra;
    };
}
