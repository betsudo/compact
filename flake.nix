# This file is part of Compact.
# Copyright (C) 2025 Midnight Foundation
# SPDX-License-Identifier: Apache-2.0
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
# 	http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

{
  description = "Compact Compiler";

  # IOG's public binary cache
  nixConfig = {
    extra-substituters = "https://cache.iog.io";
    extra-trusted-public-keys = ["hydra.iohk.io:f/Ea+s+dFdN+3Y/G+FDgSq+a5NEWhJGzdjvKNGv0/EQ="];
  };

  inputs = {
    zkir = {
      # dependency for compact-runtime release
      # this is using a tag to pull in the correct zkir version from the ledger
      # if for releasing the runtime, running nix flake update causes errors for authorization of cargo, use
      # the commit hash instead of the tag for this.
      # NOTE: if this is an internal release (uses -alpha, -beta, or -rc) do NOT update the package.json in runtime
      # since npm can only access public releases. For the compact-runtime release nix will pull in the correct
      # version from this url.
      url = "github:midnightntwrk/midnight-ledger/ledger-8.1.3"; # zkir-v2
      inputs.zkir.follows = "zkir";
    };
    n2c.url = "github:nlewo/nix2container";
    chez-exe.url = "github:tkerber/chez-exe";
    fenix = {
      url = "github:nix-community/fenix/5b3016c5f8cdd37d5e557646d369ce76e30666de";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    inclusive.url = "github:input-output-hk/nix-inclusive";
    npmlock2nix = {
      url = "github:nix-community/npmlock2nix";
      flake = false;
    };
  };

  outputs = {
    self,
    zkir,
    nixpkgs,
    utils,
    inclusive,
    chez-exe,
    fenix,
    npmlock2nix,
    ...
  } @ inputs:
    utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system}.extend (final: prev: {
          # hack to get npmlock2nix working, by pretending we're using an old
          # node version
          nodejs-16_x = final.nodejs;
          nodejs = final.nodejs_latest;
        });
        isDarwin = pkgs.lib.hasSuffix "-darwin" system;
        chez = pkgs.chez;
        sources = (import ./_sources/generated.nix) {inherit (pkgs) fetchgit fetchurl fetchFromGitHub;};
        nanopass = sources.nanopass.src;
        rough-draft = sources.rough-draft.src;
        # Cross to static musl with the native toolchain and matching target std.
        rust-linking-pkgs = if pkgs.stdenv.hostPlatform.isLinux then pkgs.pkgsStatic else pkgs;
        rust-host-target = pkgs.stdenv.hostPlatform.rust.rustcTarget;
        rust-cli-target = rust-linking-pkgs.stdenv.hostPlatform.rust.rustcTarget;
        rust = fenix.packages.${system};
        rust-version = (builtins.fromTOML (builtins.readFile ./rust-toolchain.toml)).toolchain.channel;
        rust-manifest = builtins.fromTOML (builtins.readFile (builtins.fetchurl {
          url = "https://static.rust-lang.org/dist/channel-rust-${rust-version}.toml";
          sha256 = "sha256-zm3dyIY2T414ZRR3EhLOvptzG6gta4WZUcawzMUWtqI=";
        }));
        rust-cli-toolchain = rust.combine (
          [(rust.fromManifest rust-manifest).minimalToolchain]
          ++ pkgs.lib.optional (rust-cli-target != rust-host-target)
            (rust.targets.${rust-cli-target}.fromManifest rust-manifest).rust-std
        );
        rust-cli-platform = rust-linking-pkgs.makeRustPlatform {
          cargo = rust-cli-toolchain;
          rustc = rust-cli-toolchain;
        };
        runtime-version = (__fromJSON (__readFile ./runtime/package.json)).version;
        vscode-extension-version = (__fromJSON (__readFile ./editor-support/vsc/compact/package.json)).version;
        nix2container = inputs.n2c.packages.${system}.nix2container;
        chez-exe = inputs.chez-exe.packages.${system}.default;
        runtime-shell-hook =
          ''
            rm node_modules -rf
            cp -r ${self.packages.${system}.runtime.node-modules}/node_modules node_modules
            chown $USER -R node_modules
            chmod u+w -R node_modules
            mkdir -p node_modules/@midnight-ntwrk
            cp -r ${self.packages.${system}.runtime.package}/lib/node_modules/@midnight-ntwrk/compact-runtime node_modules/@midnight-ntwrk/compact-runtime
            chown $USER -R node_modules
            chmod u+w -R node_modules
          '';
        test-center-shell-hook =
          ''
            # Set up test-center node_modules
            if [ -d test-center ]; then
              rm -rf test-center/node_modules
              cp -r ${self.packages.${system}.test-center.node-modules}/node_modules test-center/node_modules
              chown $USER -R test-center/node_modules
              chmod u+w -R test-center/node_modules
            fi
          '';
        combined-shell-hook =
          ''
            ${runtime-shell-hook}
            ${test-center-shell-hook}
          '';
        platformSpecificInputs = {
          x86_64-darwin = [ pkgs.darwin.libiconv ];
          x86_64-linux = [ pkgs.musl ];
          aarch64-darwin = [ pkgs.darwin.libiconv ];
          aarch64-linux = [ pkgs.musl ];
        }.${system};
        pretzel-js = (import nix/pretzel/js.nix) {
          inherit (nixpkgs) lib;
          inherit npmlock2nix;
          dry-install = self.packages.${system}.pretzel-dry-install;
        };
        dry-install = pretzel-js.mkPackage {
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ pretzel-js.overlay ];
          };
          src = nix/dry-install;
        };
      in
        rec {
          lib.pretzel-js = pretzel-js;
          overlays.pretzel-fetcher = (import pretzel/fetcher.nix).overlay;
          overlays.pretzel-js = lib.pretzel-js.overlay;
          packages.pretzel-dry-install = dry-install.package;
          packages.runtime = let
            inclusive-src = inclusive.lib.inclusive ./. [
              ./runtime
              ./compiler/json.ss
              ./compiler/utils.ss
              ./compiler/field.ss
              ./third_party/compiler/state-case.ss
            ];
          in lib.pretzel-js.mkPackage {
            pkgs = import nixpkgs {
              inherit system;
              overlays = [overlays.pretzel-js];
            };
            #name = "compact-runtime";
            #version = runtime-version;
            src = "${inclusive-src}/runtime";
            # Dependency installation only has the two npm manifests. The
            # runtime's prepare script runs when packing the full source.
            nodeModulesAttrs.npm_config_ignore_scripts = "true";
            overrideBuildAttrs = {
              common = oldAttrs: oldAttrs // {
                buildInputs = (oldAttrs.buildInputs or []) ++ [chez];
                CHEZSCHEMELIBDIRS = "${inclusive-src}/compiler:${inclusive-src}/third_party/compiler";
              };
            };
          };

          packages.test-center = lib.pretzel-js.mkPackage {
            pkgs = import nixpkgs {
              inherit system;
              overlays = [overlays.pretzel-js];
            };
            src = ./test-center;
          };

          packages.compactc-no-runtime = packages.compactc.overrideAttrs (oldAttrs: {
            NODE_PATH = "";
            buildInputs = [
              pkgs.nodejs
              pkgs.nodePackages.typescript
              chez
            ];
            checkPhase = "";
          });

          # The pinned root test host invokes these fixtures without compilerArgs.
          # Keep the compiler's legacy default and its explicit v3 refusals;
          # select v3 only for these byte-exact upstream compile-pass cases.
          packages.compactc-test-contracts = pkgs.writeShellScriptBin "compactc" ''
            compiler=${packages.compactc}/bin/compactc
            for arg in "$@"; do
              case "$arg" in
                --target|--target=*) exec "$compiler" "$@" ;;
              esac
            done
            for arg in "$@"; do
              expected=
              case "$arg" in
                */test-contracts/stdlib/curve25519/base/roundtrip/curve25519_base_roundtrip.compact)
                  expected=4cb617334d5ab3fd810968345cd7e564be9d9ad6c0e7616e1b2fe13fe8ab38c8
                  ;;
                */test-contracts/stdlib/curve25519/scalar/roundtrip/curve25519_scalar_roundtrip.compact)
                  expected=7e3b76271c55a71e492941398837427309d54cb32d0b33ef4660c2c6e3999c4a
                  ;;
                */test-contracts/stdlib/curve25519/default/maybe_either/curve25519_default_maybe_either.compact)
                  expected=d753887f22666a4e58ed472e282ea9d0051376230ae439ea2772824de86d279a
                  ;;
                */test-contracts/stdlib/curve25519/verify/curve25519_verify.compact)
                  expected=0208874ed5764915789f0d19d5892cd346b836d9d304f2d6e899b0c674a040c1
                  ;;
                */test-contracts/stdlib/foreign_field/self_subtract/foreign_field_self_subtract.compact)
                  expected=5a3322fc1c889390fd3a620dd905e01c96bb81808af0e7ef587ab4f5f598c26a
                  ;;
                */test-contracts/stdlib/curve25519/slow/verify_sequential/curve25519_verify_sequential.compact)
                  expected=7ad2e291ec23e496ab4f9157f1f46e17772edbe4d7516f5f2e12856a0122767f
                  ;;
                */test-contracts/stdlib/keccak256/cross_width/keccak256_cross_width.compact)
                  expected=0f57999236e7d47004e9768626f52d2efe860fef5a04439b1f7c976c3e9c9e56
                  ;;
                */test-contracts/stdlib/keccak256/adt_rejected/keccak256_adt_rejected.compact)
                  expected=92b9866f4edfa107320b97b1595102afd185b698ae4b71c02c96a7e6fb5d3378
                  ;;
                */test-contracts/stdlib/keccak256/field_types/keccak256_field_types.compact)
                  expected=319b37a50f65125de0f6b14e9c8e91cb0ecb26f27df4b20d7b6ccc6afe3ac30e
                  ;;
                */test-contracts/stdlib/keccak256/flatten_equivalence/keccak256_flatten_equivalence.compact)
                  expected=a9d34f4911f4e8faa866418d02bb0a6cbafbb9772e434331551c91da26dbca59
                  ;;
                */test-contracts/stdlib/keccak256/impure/keccak256_impure.compact)
                  expected=61f4204b918f9635828d07b6117d404ab11041bd61fd47c556b8f3cca0ab9d59
                  ;;
                */test-contracts/stdlib/keccak256/known_vectors/keccak256_known_vectors.compact)
                  expected=d95c93c2abddfbee69dae5b9de6b368ae9af6baff214367d3e18c16bf91767b3
                  ;;
                */test-contracts/stdlib/keccak256/multi_length/keccak256_multi_length.compact)
                  expected=3d6cd76f04078b7cc4259580b1e7c093a9816752b56651672bc773c557bafeb2
                  ;;
                */test-contracts/stdlib/keccak256/ordinary_types/keccak256_ordinary_types.compact)
                  expected=66856b94eb03bfe4eaade139cc3dd966bc6aaad9f9d4a4359c1fc7c551667988
                  ;;
                */test-contracts/stdlib/keccak256/trailing_zero/keccak256_trailing_zero.compact)
                  expected=d8e791a46417c717ef51ffb155657c07660ef19ceeeb755b1481515e804428f1
                  ;;
                */test-contracts/stdlib/secp256k1/bitcoin_kat/secp256k1_bitcoin_kat.compact)
                  expected=3f600e9c25498b8cd6c8b82270286aa58d864f1fe9a7e10ee1a01315c8c9b086
                  ;;
                */test-contracts/stdlib/secp256k1/base/roundtrip/secp256k1_base_roundtrip.compact)
                  expected=c05e619523818a4bc6e870c367516edfc9fc8c42b4e0236c3f4687661e3a1548
                  ;;
                */test-contracts/stdlib/secp256k1/ecdsa_verify/secp256k1_ecdsa_verify.compact)
                  expected=83aaf4309bbaa741537371d6d23ea55b46282abd892c255075c0b60fb3b16462
                  ;;
                */test-contracts/stdlib/secp256k1/ethereum_address/secp256k1_ethereum_address.compact)
                  expected=819c1d11c841f15fa00ca01cabfbb8e659c8bbb41cd7ba12e20c88c5303007c2
                  ;;
                */test-contracts/stdlib/secp256k1/slow/prove_ethereum_address/secp256k1_prove_ethereum_address.compact)
                  expected=14c198a3d1df93579a176de06b31af10c6062d03b2eae5bc2c7a19f6d58bc459
                  ;;
                */test-contracts/stdlib/secp256k1/slow/verify_sequential/secp256k1_verify_sequential.compact)
                  expected=e1ba0175af4e5ff91af065e87dc631e89f033195177436cfc2c386331a68f3fe
                  ;;
                */test-contracts/stdlib/secp256r1/ecdsa_verify/secp256r1_ecdsa_verify.compact)
                  expected=ce45565985916e21c16ec14ed351a05eaa44da144d08ff30fde682dcc646db30
                  ;;
                */test-contracts/stdlib/secp256r1/base/roundtrip/secp256r1_base_roundtrip.compact)
                  expected=9f2a45968d342ef39ead91389665df952925f6e31bb725fdce2bf1f862bef9f9
                  ;;
                */test-contracts/stdlib/secp256r1/sha256_kat/secp256r1_sha256_kat.compact)
                  expected=d174d86cf6e3af7be6d5a05485c8c3a39fb1b202f8e467e06fc1b8785e46540e
                  ;;
                */test-contracts/stdlib/secp256r1/slow/verify_sequential/secp256r1_verify_sequential.compact)
                  expected=f1c20ef9040f20fc347007dda991b173e21e8d817d24a01e3bd41a9f7202c665
                  ;;
                */test-contracts/stdlib/sha256/multi_length/sha256_multi_length.compact)
                  expected=1adb85bdf1d1be98dbf2635564011437b3a9366d96be467f2e82864314efccc0
                  ;;
                */test-contracts/stdlib/sha256/trailing_zero/sha256_trailing_zero.compact)
                  expected=96a514e793a1870ef03b0d25e1efbb84d681d793af9b5ae8435c30bb8ad88636
                  ;;
              esac
              if [ -n "$expected" ]; then
                if ! { [ "$#" -eq 2 ] && [ "$1" = "$arg" ]; } &&
                   ! { [ "$#" -eq 3 ] && [ "$1" = --skip-zk ] && [ "$2" = "$arg" ]; }; then
                  echo "compactc-test-contracts: unexpected arguments for pinned fixture" >&2
                  exit 1
                fi
                digest=$(${pkgs.coreutils}/bin/sha256sum "$arg" | ${pkgs.coreutils}/bin/cut -d ' ' -f 1)
                if [ "$digest" != "$expected" ]; then
                  echo "compactc-test-contracts: pinned fixture hash changed" >&2
                  exit 1
                fi
                exec "$compiler" --feature-zkir-v3 "$@"
              fi
            done
            exec "$compiler" "$@"
          '';

          packages.compact-rust-cli = rust-cli-platform.buildRustPackage ({
            pname = "compact-rust-cli";
            version = "0.2.0";
            # The CLI build needs only its crate sources; local harness scripts
            # and oracle captures must not force rebuilding the release binary.
            src = inclusive.lib.inclusive ./tools/compact-rust-backend [
              ./tools/compact-rust-backend/Cargo.toml
              ./tools/compact-rust-backend/Cargo.lock
              ./tools/compact-rust-backend/README.md
              ./tools/compact-rust-backend/src
            ];
            cargoLock = {
              lockFile = ./tools/compact-rust-backend/Cargo.lock;
              # The packages below come from two immutable Ledger/ZK
              # checkouts. Nix still requires the shared checkout hash for
              # each package identity referenced by Cargo.lock.
              outputHashes = {
                "midnight-base-crypto-1.1.1" = "sha256-7nPujsILZToXHIBcRemu8NLrxmrg2+QM4sKPsJwGgu0=";
                "midnight-base-crypto-derive-1.1.0" = "sha256-7nPujsILZToXHIBcRemu8NLrxmrg2+QM4sKPsJwGgu0=";
                "midnight-serialize-1.2.1" = "sha256-7nPujsILZToXHIBcRemu8NLrxmrg2+QM4sKPsJwGgu0=";
                "midnight-serialize-macros-1.0.0" = "sha256-7nPujsILZToXHIBcRemu8NLrxmrg2+QM4sKPsJwGgu0=";
                "midnight-storage-core-1.2.3" = "sha256-7nPujsILZToXHIBcRemu8NLrxmrg2+QM4sKPsJwGgu0=";
                "midnight-storage-macros-1.0.1" = "sha256-7nPujsILZToXHIBcRemu8NLrxmrg2+QM4sKPsJwGgu0=";
                "midnight-transient-crypto-3.1.0" = "sha256-7nPujsILZToXHIBcRemu8NLrxmrg2+QM4sKPsJwGgu0=";
                "midnight-circuits-7.3.0" = "sha256-lofqa58hDlJ0C3vIWFodsdwYYwYoPEDpS1PMGV9TULg=";
                "midnight-curves-0.3.1" = "sha256-lofqa58hDlJ0C3vIWFodsdwYYwYoPEDpS1PMGV9TULg=";
                "midnight-proofs-0.8.2" = "sha256-lofqa58hDlJ0C3vIWFodsdwYYwYoPEDpS1PMGV9TULg=";
                "midnight-zk-stdlib-2.4.0" = "sha256-lofqa58hDlJ0C3vIWFodsdwYYwYoPEDpS1PMGV9TULg=";
              };
            };
            cargoBuildFlags = [ "--bin" "compactc" ];
            # Compiler-backed gates run in build-compiler.yml against the full
            # workspace; this derivation packages just the public command.
            doCheck = false;
            postInstall = ''
              mkdir -p $out/share/compactc
              cp -R ${./runtime-rs} $out/share/compactc/runtime-rs
              cp -R ${./runtime-rs-macros} $out/share/compactc/runtime-rs-macros
            '';
          } // pkgs.lib.optionalAttrs (pkgs.lib.hasSuffix "linux" system) {
            postFixup = ''
              ${pkgs.buildPackages.binutils}/bin/readelf -lW "$out/bin/compactc" > "$TMPDIR/compact-rust-cli-program-headers" || exit 1
              ${pkgs.buildPackages.binutils}/bin/readelf -dW "$out/bin/compactc" > "$TMPDIR/compact-rust-cli-dynamic-section" || exit 1
              if ${pkgs.buildPackages.gnugrep}/bin/grep -Eq '(^|[[:space:]])INTERP([[:space:]]|$)' "$TMPDIR/compact-rust-cli-program-headers"; then
                echo "compact-rust-cli must not require an ELF interpreter" >&2
                exit 1
              else
                test "$?" -eq 1 || exit 1
              fi
              if ${pkgs.buildPackages.gnugrep}/bin/grep -Fq '(NEEDED)' "$TMPDIR/compact-rust-cli-dynamic-section"; then
                echo "compact-rust-cli must not require shared libraries" >&2
                exit 1
              else
                test "$?" -eq 1 || exit 1
              fi
              echo "compact-rust-cli: static ELF verified for ${system}"
            '';
          });

          packages.compactc = pkgs.stdenv.mkDerivation {
            name = "compactc";
            version = "0.31.220"; # NB: also update compiler-version in compiler/compiler-version.ss
            src = inclusive.lib.inclusive ./. [
              ./compiler
              ./examples
              ./flake.nix
              ./runtime/extract-version.ss
              ./runtime/package.json
              ./srcMaps
              ./test-center
              ./test-contracts/stdlib/historic_merkle_tree/deep_default
              ./test-contracts/stdlib/merkle_tree/deep_default
              ./third_party/compiler
            ];

            CHEZSCHEMELIBDIRS = "compiler::obj/compiler:third_party/compiler::obj/third_party/compiler:${nanopass}::obj/nanopass:${rough-draft}/src::obj/rough-draft:srcMaps::obj/srcMaps::obj/compiler";

            NODE_PATH = "${packages.runtime.node-modules}/node_modules";

            buildInputs = [
              pkgs.nodejs
              pkgs.nodePackages.typescript
              packages.runtime.package
              packages.runtime.node-modules
              chez
            ];

            buildPhase = ''
              mkdir -p obj/compiler
              sed -e 's;/usr/bin/env .*;'`command -v scheme`' --program;' compiler/compactc.ss > obj/compiler/compactc.ss
              sed -e 's;/usr/bin/env .*;'`command -v scheme`' --program;' compiler/format-compact.ss > obj/compiler/format-compact.ss
              sed -e 's;/usr/bin/env .*;'`command -v scheme`' --program;' compiler/fixup-compact.ss > obj/compiler/fixup-compact.ss
              patchShebangs --host .

              scheme -q << END
                (reset-handler abort)
                (optimize-level 2)
                (compile-imported-libraries #t)
                (generate-wpo-files #t)
                (generate-inspector-information #f)
                (compile-profile #f)
                (compile-program "obj/compiler/compactc.ss" "obj/compiler/compactc.so")
                (compile-program "obj/compiler/format-compact.ss" "obj/compiler/format-compact.so")
                (compile-program "obj/compiler/fixup-compact.ss" "obj/compiler/fixup-compact.so")
                (compile-whole-program "obj/compiler/compactc.wpo" "obj/compactc")
                (compile-whole-program "obj/compiler/format-compact.wpo" "obj/format-compact")
                (compile-whole-program "obj/compiler/fixup-compact.wpo" "obj/fixup-compact")
              END
            '';

            # check if the code was build correctly
            checkPhase = ''
              cp -r ${packages.runtime.node-modules}/node_modules node_modules
              chmod -R +rw node_modules
              mkdir -p node_modules/@midnight-ntwrk
              cp -r ${packages.runtime.package}/lib/node_modules/@midnight-ntwrk/compact-runtime node_modules/@midnight-ntwrk/compact-runtime
              ./compiler/go
              ./srcMaps/test.sh
            '';

            installPhase = ''
              mkdir -p $out/bin
              cp obj/compactc $out/bin/compactc-scheme
              cp ${packages.compact-rust-cli}/bin/compactc $out/bin/compactc
              mkdir -p $out/share
              cp -R ${packages.compact-rust-cli}/share/compactc $out/share/compactc
              cp obj/format-compact $out/bin
              cp obj/fixup-compact $out/bin
              chmod +x $out/bin/compactc $out/bin/compactc-scheme
              chmod +x $out/bin/format-compact
              chmod +x $out/bin/fixup-compact
            '';
          };

          packages.compactc-binary-nixos = let
            linking-pkgs = if pkgs.lib.hasSuffix "linux" system then pkgs.pkgsMusl else pkgs;
            platformSpecificInputs = with linking-pkgs; {
              x86_64-darwin = [ darwin.libiconv ];
              x86_64-linux = [ ];
              aarch64-darwin = [ darwin.libiconv ];
              aarch64-linux = [ ];
            }.${system};
          in linking-pkgs.stdenv.mkDerivation {
            name = "compactc-binary-nixos";
            version = "0.0.1";
            src = inclusive.lib.inclusive ./. [
              ./compiler
              ./examples
              ./flake.nix
              ./runtime/extract-version.ss
              ./runtime/package.json
              ./srcMaps
              ./test-center
              ./third_party/compiler
            ];

            CHEZSCHEMELIBDIRS = "compiler::obj/compiler:third_party/compiler::obj/third_party/compiler:${nanopass}::obj/nanopass:${rough-draft}/src::obj/rough-draft:srcMaps::obj/srcMaps::obj/compiler";

            buildInputs = [
              chez-exe
            ] ++ platformSpecificInputs;

            buildPhase = ''
              mkdir -p obj/compiler

              sed -e 's;/usr/bin/env .*;'`command -v scheme`' --program;' compiler/compactc.ss > obj/compiler/compactc.ss
              sed -e 's;/usr/bin/env .*;'`command -v scheme`' --program;' compiler/format-compact.ss > obj/compiler/format-compact.ss
              sed -e 's;/usr/bin/env .*;'`command -v scheme`' --program;' compiler/fixup-compact.ss > obj/compiler/fixup-compact.ss

              compile-chez-program --optimize-level 2 obj/compiler/compactc.ss
              compile-chez-program --optimize-level 2 obj/compiler/format-compact.ss
              compile-chez-program --optimize-level 2 obj/compiler/fixup-compact.ss
            '';

            installPhase = ''
              mkdir -p $out/bin

              cp obj/compiler/compactc $out/bin/compactc-scheme
              cp ${packages.compact-rust-cli}/bin/compactc $out/bin/compactc
              mkdir -p $out/share
              cp -R ${packages.compact-rust-cli}/share/compactc $out/share/compactc
              chmod +x $out/bin/compactc $out/bin/compactc-scheme

              for exe in format-compact fixup-compact; do
                cp "obj/compiler/$exe" $out/bin
                chmod +x "$out/bin/$exe"
              done
            '' + (if isDarwin then ''
              for exe in compactc compactc-scheme format-compact fixup-compact; do
                install_name_tool -change ${pkgs.darwin.libiconv}/lib/libiconv.2.dylib /usr/lib/libiconv.2.dylib "$out/bin/$exe"
              done
            '' else "");
          };

          packages.zkir-v3 = rust-cli-platform.buildRustPackage rec {
            pname = "midnight-zkir-v3";
            version = (builtins.fromJSON (builtins.readFile ./test-center/package.json)).dependencies."@midnightntwrk/zkir-v3";
            src = pkgs.fetchurl {
              url = "https://static.crates.io/crates/midnight-zkir/midnight-zkir-${version}.crate";
              name = "midnight-zkir-${version}.tar.gz";
              hash = "sha256-r5jDrGGPRKYJ30lj20vmm+10EjoxEbVHXiKoHOxK2Fc=";
            };
            cargoLock.lockFile = ./nix/zkir-v3/Cargo.lock;
            postPatch = ''
              cp ${cargoLock.lockFile} Cargo.lock
            '';
            cargoBuildFlags = [ "--locked" "--bin" "zkir" "--features" "binary" ];
            doCheck = false;
          };

          # The upstream zkir-v3 package produces bin/zkir (same name as
          # zkir v2).  Wrap it so the binary is available as bin/zkir-v3,
          # which is the name the compiler invokes.
          packages.zkir-v3-bin = pkgs.runCommand "zkir-v3-bin" {} ''
            mkdir -p $out/bin
            ln -s ${packages.zkir-v3}/bin/zkir $out/bin/zkir-v3
          '';

          packages.compactc-binaryWrapperScript-nixos = pkgs.writeShellScriptBin "run-compactc" ''
            PATH=${pkgs.lib.makeBinPath [ packages.compactc-binary-nixos zkir.packages.${system}.zkir packages.zkir-v3-bin ]} \
            compactc "$@"
          '';

          packages.compactc-binary = pkgs.stdenv.mkDerivation {
            name = "compactc-binary-dist";
            version = "0.0.1";
            src = packages.compactc-binary-nixos;

            installPhase = ''
              mkdir -p $out/bin $out/lib
              cp -R share $out/share
              cp bin/compactc $out/bin
              mv $out/bin/compactc $out/bin/compactc.bin
              cp bin/compactc-scheme $out/bin/compactc-scheme
              cp ${zkir.packages.${system}.zkir}/bin/zkir $out/lib/zkir
              cp ${packages.zkir-v3}/bin/zkir $out/lib/zkir-v3

              chmod +w $out/lib/zkir
              chmod +w $out/lib/zkir-v3

              cp ${./tools/compact-rust-backend/compactc-launcher.sh} $out/bin/compactc
              chmod +x $out/bin/compactc

              for exe in format-compact fixup-compact; do
                cp "bin/$exe" $out/bin
                chmod +x "$out/bin/$exe"
              done
            '' + (if isDarwin then ''
              install_name_tool -change ${inputs.zkir.inputs.nixpkgs.legacyPackages.${system}.darwin.libiconv}/lib/libiconv.2.dylib /usr/lib/libiconv.2.dylib "$out/lib/zkir"
              install_name_tool -change ${pkgs.darwin.libiconv}/lib/libiconv.2.dylib /usr/lib/libiconv.2.dylib "$out/lib/zkir-v3"
            '' else "");

            # Execution on a Nix-equipped host can hide store-only libraries.
            # Fail the portable Darwin build if any executable still needs one.
            doInstallCheck = isDarwin;
            installCheckPhase = ''
              for exe in bin/compactc.bin bin/compactc-scheme bin/format-compact bin/fixup-compact lib/zkir lib/zkir-v3; do
                otool -L "$out/$exe" > "$TMPDIR/compactc-libraries"
                if grep -Eq '^[[:space:]]+/nix/store/' "$TMPDIR/compactc-libraries"; then
                  cat "$TMPDIR/compactc-libraries" >&2
                  echo "Portable compiler retains a Nix library dependency: $exe" >&2
                  exit 1
                fi
              done
            '';

            dontFixup = true;
          };

          packages.compactc-oci = with pkgs // packages; nix2container.buildImage {
            name = "compactc";
            tag = compactc.version;
            copyToRoot = [
              # When we want tools in /, we need to symlink them in order to
              # still have libraries in /nix/store. This behavior differs from
              # dockerTools.buildImage but this allows to avoid having files
              # in both / and /nix/store.
              (pkgs.buildEnv {
                name = "root";
                paths = [
                  bashInteractive
                  coreutils
                ];
                pathsToLink = [ "/bin" ];
              })
            ];

            config = {
              entrypoint = [ "${bashInteractive}/bin/bash" "-c" ];
              Cmd = [ "${compactc}/bin/compactc" ];
              Env = [
                "PATH=${pkgs.lib.makeBinPath [
                  compactc
                  zkir.packages.${system}.zkir
                  zkir-v3-bin
                ]}"
              ];
            };
            layers = [
              (nix2container.buildLayer {
                deps = [
                  compactc
                  zkir.packages.${system}.zkir
                  zkir-v3-bin
                ];
              })
            ];
          };

          packages.compact-vscode-extension-node-modules = pkgs.mkYarnModules {
            pname = "compact-vscode-extension-node-modules";
            version = vscode-extension-version;
            packageJSON = ./editor-support/vsc/compact/package.json;
            yarnLock = ./editor-support/vsc/compact/yarn.lock;
          };

          packages.compact-vscode-extension =
            pkgs.lib.customisation.overrideDerivation (pkgs.stdenv.mkDerivation rec {
              pname = "compact-vscode-extension";
              version = vscode-extension-version;
              src = ./editor-support/vsc/compact;
              buildInputs = with pkgs; [
                nodejs
                yarn
              ];
              nativeBuildInputs = [
                 chez
                 packages.compactc
              ];

              TEST_COMPACT_PATH = ./test-center/compact/test.compact;
              CHEZSCHEMELIBDIRS = "${nanopass}::obj/nanopass:${rough-draft}/src::obj/rough-draft:srcMaps::obj/srcMaps:compiler::obj/compiler:third_party/compiler::obj/third_party/compiler";

              buildPhase = ''
                ln -s ${packages.compact-vscode-extension-node-modules}/node_modules
              '';

              doCheck = true;

              checkPhase = ''
                echo Get compiler
                mkdir -p tmp/obj
                cp -R ${packages.compactc.src}/* tmp
                cd tmp

                echo Export current keywords
                scheme --program ./compiler/export-keywords.ss $(pwd)/../tests/resources/keywords.json
                cat $(pwd)/../tests/resources/keywords.json
                cd ..

                echo Run unit tests
                yarn run --offline test --ci --reporters=jest-silent-reporter --reporters=summary
              '';

            installPhase = ''
              mkdir -p $out
              yarn build
              yarn vsce package --yarn -o $out
            '';
          })
           (drv: {
              extensionFile = drv.outPath + "/compact-${drv.version}.vsix";
            });

          packages.test-src-maps = pkgs.lib.customisation.overrideDerivation (pkgs.stdenv.mkDerivation {
            pname = "compact-source-maps-tests";
            version = vscode-extension-version;
            src = ./test-src-maps;
            buildInputs = with pkgs; [
              nodejs
              yarn
            ];

            buildPhase = ''
              yarn --offline build
            '';

            doCheck = true;

            checkPhase = ''
              yarn run --offline test
            '';

            installPhase = ''
              mkdir -p $out
              touch $out/ok
            '';
          });

          packages.all = pkgs.symlinkJoin {
            name = "compact-all";
            meta.mainProgram = "compactc";
            paths = [
              packages.compactc
              zkir.packages.${system}.zkir
              packages.zkir-v3-bin
              packages.compact-vscode-extension
            ];
          };

          packages.default = packages.all;

          devShells.default = pkgs.mkShell {
            inputsFrom = with packages; [compactc];
            packages = [
              pkgs.nodejs
              pkgs.yarn
              pkgs.python3
              pkgs.alejandra
              packages.runtime.package
              packages.runtime.node-modules
              packages.test-center.package
              packages.test-center.node-modules
            ];
            shellHook = combined-shell-hook;

            CHEZSCHEMELIBDIRS = "compiler::obj/compiler:third_party/compiler::obj/third_party/compiler:${nanopass}::obj/nanopass:${rough-draft}/src::obj/rough-draft:srcMaps::obj/srcMaps";
            WASM_BINDGEN_WEAKREF = 1;
            WASM_BINDGEN_EXTERNREF = 1;
          };

          devShells.with-zkir = packages.runtime.mkShell {
            inputsFrom = with packages; [compactc];
            packages = [
              pkgs.nodejs
              pkgs.yarn
              pkgs.python3
              pkgs.binaryen
              packages.runtime.package
              packages.runtime.node-modules
              packages.test-center.package
              packages.test-center.node-modules
              zkir.packages.${system}.zkir
              packages.zkir-v3-bin
            ];
            shellHook = combined-shell-hook;

            CHEZSCHEMELIBDIRS = "compiler::obj/compiler:third_party/compiler::obj/third_party/compiler:${nanopass}::obj/nanopass:${rough-draft}/src::obj/rough-draft:srcMaps::obj/srcMaps";
          };

          devShells.compiler = pkgs.mkShell {
            inputsFrom = with packages; [compactc];
            packages = [
              packages.compactc
              pkgs.yarn
              pkgs.python3
              zkir.packages.${system}.zkir
              packages.zkir-v3-bin
            ];

            CHEZSCHEMELIBDIRS = "compiler::obj/compiler:third_party/compiler::obj/third_party/compiler:${nanopass}::obj/nanopass:${rough-draft}/src::obj/rough-draft:srcMaps::obj/srcMaps";
          };

          devShells.runtime = packages.runtime.mkShell {
            packages = [
              pkgs.nodejs
              pkgs.chez
            ];
            shellHook = runtime-shell-hook;
          };

          # The pinned root wrapper needs both the guarded compiler and a
          # built runtime package with declarations for generated imports.
          devShells.test-contracts = pkgs.mkShell {
            packages = [
              packages.compactc-test-contracts
              packages.runtime.package
              pkgs.nodejs
            ];
            COMPACT_RUNTIME_PKG = "${packages.runtime.package}/lib/node_modules/@midnight-ntwrk/compact-runtime";
          };

          devShells.dapp = packages.runtime.mkShell {
            packages = [
              packages.compactc
              packages.runtime.package
              packages.runtime.node-modules
              zkir.packages.${system}.zkir
              packages.zkir-v3-bin
              pkgs.nodejs
              pkgs.yarn
              pkgs.python3
            ];
            shellHook = runtime-shell-hook;
          };

          formatter = pkgs.alejandra;
        }
    );
}
