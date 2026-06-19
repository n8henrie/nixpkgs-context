# nixpkgs-context

master: [![master branch build status](https://github.com/n8henrie/nixpkgs-context/actions/workflows/ci.yml/badge.s
vg?branch=master)](https://github.com/n8henrie/nixpkgs-context/actions/workflows/ci.yml)

```console
$ target/release/nixpkgs-context --help
Find nix code examples in context for a given string

Usage: nixpkgs-context [OPTIONS] <NEEDLE> [PATH]

Arguments:
  <NEEDLE>
  [PATH]    directory name to (recursively) search for nix files [default: .]

Options:
  -e, --examples <EXAMPLES>  number of examples to show per binding [default: 0]
  -h, --help                 Print help
  -V, --version              Print version
$
$ target/release/nixpkgs-context pkg-config
nativeBuildInputs (1)
$
$ target/release/nixpkgs-context pkg-config -e 2
nativeBuildInputs (1)
=== EXAMPLES ===
./tests/files/garage/default.nix:
    nativeBuildInputs = [
        protobuf
        pkg-config
      ];
$
```
