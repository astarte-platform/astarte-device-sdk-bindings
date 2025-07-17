{
  description = "A Nix-flake-based Python development environment";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.astarte-sdk-python = {
    url = "github:astarte-platform/astarte-device-sdk-python";
    flake = false;
  };

  outputs = { self, nixpkgs, astarte-sdk-python }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forEachSupportedSystem = f: nixpkgs.lib.genAttrs supportedSystems (system: f {
        pkgs = nixpkgs.legacyPackages.${system};
      });
    in
    {
      devShells = forEachSupportedSystem ({ pkgs }: let

    astarte-message-hub-proto = ps:
       ps.buildPythonPackage rec {
        pname = "astarte-message-hub-proto";
        version = "0.7.0";

        format = "pyproject";

        src = ps.fetchPypi {
          inherit version;
          pname = nixpkgs.lib.strings.stringAsChars (c: if c == "-" then "_" else c) pname;
          sha256 = "pSGLdgu6yOE2D2e9LL+RJzecqp0Gw5sdnx6XwXCCuYg=";
        };

        dependencies = with ps; [
          setuptools
          grpcio
          protobuf
        ];
      };

    astarte-device-sdk = ps:
      ps.buildPythonPackage {
        pname = "astarte-device-sdk";
        version = "0.13.3";

        format = "pyproject";

        src = astarte-sdk-python;

        buildInputs = with ps; [
          setuptools
        ];

        dependencies = with ps; [
          setuptools
          (astarte-message-hub-proto ps)
          requests
          paho-mqtt
          cryptography
          bson
          pyjwt
        ];
      };

        pythonEnv = pkgs.python3.withPackages (pythonPkgs: with pythonPkgs; [
          pip
          venvShellHook
          setuptools
          wheel
          #(astarte-device-sdk pythonPkgs)
        ]);

      in {
        default = pkgs.mkShell rec {
          venvDir = ".venv";
          buildInputs = [
            pkgs.autoPatchelfHook
          ];
          packages = with pkgs; [
            pythonEnv
            black
            pyright
            virtualenv
            isort
            # rust sdk related stuff
            rustup
            sqlite
          ];
          shellHook = ''
            if [ ! -d ${venvDir} ]; then
              echo "creating venv"
              virtualenv ./${venvDir}
              pip install pytest
              pip install pylint
              autoPatchelf ./${venvDir}
              source ./${venvDir}/bin/activate
            else
              echo "activating venv"
              autoPatchelf ./${venvDir}
              source ./${venvDir}/bin/activate
            fi
          '';
        };
      });
    };
}
