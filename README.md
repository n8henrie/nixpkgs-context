# nixpkgs-context

master: [![master branch build status](https://github.com/n8henrie/nixpkgs-context/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/n8henrie/nixpkgs-context/actions/workflows/ci.yml)

## Overview

This project is scratching a personal itch, helping answer questions like:

- "where do most nixpkgs contributors put `pkg-config` (answer: `nativeBuildInputs`)
- "in what phase should I put `patchShebangs`?" (answer: likely `postPatch`)

I'm a relative novice when it comes to nix (and rust for that matter), but I like trying to contribute to its ecosystem.
I often wonder whether a step should conventionally go in e.g. `installPhase` or maybe `postFixup` or should it be `preConfigure`?

`nixpkgs-context` helps answer this by recursively scanning all nix files in the directory `PATH` (default `.`) that contain the argument passed in as `NEEDLE`.
It then tries to parse those nix files with tree-sitter and determine the binding that included that word.

It optionally provides a configurable number of examples, including the path where these were found.

## Usage

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
$ target/release/nixpkgs-context pkg-config -e 2 ~/git/nixpkgs | head -30
nativeBuildInputs (6865)
=== EXAMPLES ===
/Users/n8henrie/git/nixpkgs/pkgs/servers/sql/mysql/8.0.x.nix:
    nativeBuildInputs = [
    bison
    cmake
    pkg-config
    protobuf
  ]
  ++ lib.optionals (!stdenv.hostPlatform.isDarwin) [ rpcsvc-proto ];
/Users/n8henrie/git/nixpkgs/pkgs/servers/sql/percona-server/8_0.nix:
    nativeBuildInputs = [
    bison
    cmake
    pkg-config
    makeWrapper
    # required for scripts/CMakeLists.txt
    coreutils
    gnugrep
    procps
  ]
  ++ lib.optionals (!stdenv.hostPlatform.isDarwin) [ rpcsvc-proto ];

depsBuildBuild (177)
=== EXAMPLES ===
/Users/n8henrie/git/nixpkgs/pkgs/os-specific/linux/kernel/generic.nix:
    depsBuildBuild =
              previousAttrs.depsBuildBuild or [ ]
              ++ (with pkgsBuildBuild; [
                pkg-config
$
```

## Performance

On my M4 Max, searching a full `nixpkgs` checkout for `patchShebangs`:

```console
$ hyperfine --runs 100 'target/release/nixpkgs-context patchShebangs ~/git/nixpkgs'
Benchmark 1: target/release/nixpkgs-context patchShebangs ~/git/nixpkgs
  Time (mean ± σ):      2.414 s ±  0.073 s    [User: 0.616 s, System: 21.215 s]
  Range (min … max):    2.185 s …  2.559 s    100 runs
```

## Contributing

Contributions are welcome!
I *strongly* recommend that potential contributors start an issue for discussion prior to embarking on any nontrivial effort; I am somewhat picky about what free puppies I adopt.

In general:

- performance is important, but not as important as readability and safety
- no new 3rd-party dependencies without prior discussion
- new features should include tests
- bug fixes should include regression tests

## TODO

- benchmarking
- more and better tests
