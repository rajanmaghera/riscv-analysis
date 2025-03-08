{
  description = "RISC-V analysis flake";

  inputs = {
    fenix.url = "github:nix-community/fenix";
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = { self, nixpkgs, fenix, naersk }:
    let
      buildTargets = {
        "x86_64-darwin" = {
          crossSystemConfig = "x86_64-apple-darwin";
          rustTarget = "x86_64-apple-darwin";
        };

        "aarch64-darwin" = {
          crossSystemConfig = "aarch64-apple-darwin";
          rustTarget = "aarch64-apple-darwin";
        };

        "x86_64-linux" = {
          crossSystemConfig = "x86_64-unknown-linux-musl";
          rustTarget = "x86_64-unknown-linux-musl";
        };

        "aarch64-linux" = {
          crossSystemConfig = "aarch64-unknown-linux-musl";
          rustTarget = "aarch64-unknown-linux-musl";
        };
      };

      eachSystem = supportedSystems: callback: builtins.foldl'
        (overall: system: overall // { ${system} = callback system; })
        {}
        supportedSystems;

      eachCrossSystem = supportedSystems: callback:
        eachSystem supportedSystems (buildSystem: builtins.foldl'
            (inner: targetSystem: inner // {
              "cross-${targetSystem}" = callback buildSystem targetSystem;
            })
            { default = callback buildSystem buildSystem; }
            supportedSystems
        );

      mkPkgs = buildSystem: targetSystem: import nixpkgs ({
        system = buildSystem;
      } // (if targetSystem == null then {} else {
        # The nixpkgs cache doesn't have any packages where cross-compiling has
        # been enabled, even if the target platform is actually the same as the
        # build platform (and therefore it's not really cross-compiling). So we
        # only set up the cross-compiling config if the target platform is
        # different.
        crossSystem.config = buildTargets.${targetSystem}.crossSystemConfig;
      }));

      truePackages = (eachCrossSystem
        (builtins.attrNames buildTargets)
        (buildSystem: targetSystem: let
          pkgs = mkPkgs buildSystem null;
          pkgsCross = mkPkgs buildSystem targetSystem;
          rustTarget = buildTargets.${targetSystem}.rustTarget;

          fenixPkgs = fenix.packages.${buildSystem};

          mkToolchain = fenixPkgs: fenixPkgs.toolchainOf {
            channel = "stable";
            date = "2025-02-20";
            sha256 = "sha256-AJ6LX/Q/Er9kS15bn9iflkUwcgYqRQxiOIL2ToVAXaU=";
          };

          toolchain = fenixPkgs.combine [
            (mkToolchain fenixPkgs).rustc
            (mkToolchain fenixPkgs).cargo
            (mkToolchain fenixPkgs.targets.${rustTarget}).rust-std
          ];

          buildPackageAttrs = if
            builtins.hasAttr "makeBuildPackageAttrs" buildTargets.${targetSystem}
          then
            buildTargets.${targetSystem}.makeBuildPackageAttrs pkgsCross
          else
            {};

          naersk-lib = pkgs.callPackage naersk {
            cargo = toolchain;
            rustc = toolchain;
          };

        in
          naersk-lib.buildPackage (buildPackageAttrs // rec {
            src = ./.;
            version = "0.1.0-alpha.1";
            cargoBuildOptions = x: x ++ [ "-p" "riscv_analysis_cli" ];
            cargoTestOptions = x: x ++ [ "-p" "riscv_analysis_cli" ];
            strictDeps = true;
            doCheck = false;
            TARGET_CC = "${pkgsCross.stdenv.cc}/bin/${pkgsCross.stdenv.cc.targetPrefix}cc";
            CARGO_BUILD_TARGET = rustTarget;
            CARGO_BUILD_RUSTFLAGS = [
              "-C" "target-feature=+crt-static"
              "-C" "linker=${TARGET_CC}"
            ];
          })
        ));
        in {
          packages = truePackages
           // {
          "aarch64-darwin" = let
           version = truePackages."aarch64-darwin".default.version;
           in
          {

            "my-file" = nixpkgs.legacyPackages.aarch64-darwin.runCommand "my-file" {}
              ''
              mkdir -p $out/RVA/Linux-x86_64/latest
              mkdir -p $out/RVA/Linux-aarch64/latest
              mkdir -p $out/RVA/Darwin-x86_64/latest
              mkdir -p $out/RVA/Darwin-aarch64/latest
              mkdir -p $out/RVA/Linux-x86_64/${version}
              mkdir -p $out/RVA/Linux-aarch64/${version}
              mkdir -p $out/RVA/Darwin-x86_64/${version}
              mkdir -p $out/RVA/Darwin-aarch64/${version}
              cp ${truePackages."aarch64-darwin".cross-aarch64-linux}/bin/rva $out/RVA/Linux-aarch64/${version}/rva
              cp ${truePackages."aarch64-darwin".cross-x86_64-linux}/bin/rva $out/RVA/Linux-x86_64/${version}/rva
              cp ${truePackages."aarch64-darwin".cross-aarch64-darwin}/bin/rva $out/RVA/Darwin-aarch64/${version}/rva
              cp ${truePackages."aarch64-darwin".cross-x86_64-darwin}/bin/rva $out/RVA/Darwin-x86_64/${version}/rva
              ln -s --relative $out/RVA/Linux-x86_64/${version}/rva $out/RVA/Linux-x86_64/latest/rva
              ln -s --relative $out/RVA/Linux-aarch64/${version}/rva $out/RVA/Linux-aarch64/latest/rva
              ln -s --relative $out/RVA/Darwin-x86_64/${version}/rva $out/RVA/Darwin-x86_64/latest/rva
              ln -s --relative $out/RVA/Darwin-aarch64/${version}/rva $out/RVA/Darwin-aarch64/latest/rva
              '';

            myexample = (nixpkgs.legacyPackages.aarch64-darwin.symlinkJoin { name = "myexample"; paths = [
              truePackages."aarch64-darwin".cross-aarch64-linux
              truePackages."aarch64-darwin".cross-aarch64-darwin
              truePackages."aarch64-darwin".cross-x86_64-linux
              truePackages."aarch64-darwin".cross-x86_64-darwin
             ]; postBuild = "echo links added"; });
          };
        };

    };

}
