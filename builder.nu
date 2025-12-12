#!/usr/bin/env -S nu --stdin
use std/log

def main [] {}

def "main vendor" [
  src: string
] {
  print $env

  let cargo_home = $"($env.TMP)/cargo_home"
  mkdir $cargo_home

  cd $src
  CARGO_HOME=$cargo_home cargo vendor $env.out -q
}

def "main build" [
  src: string
  deps: string
  package: string
  cargo_config: string
] {
  print $env

  let out = $env.out
  let cargo_target = $"($out)/cargo_target"
  let cargo_home = $"($out)/cargo_home"

  mkdir $cargo_home

  $cargo_config | from toml | upsert source.vendored-sources.directory $deps | save $"($cargo_home)/config.toml"

  cd $src
  mkdir $cargo_target
  mkdir $"($out)/bin"
  CARGO_HOME=$cargo_home CARGO_TARGET_DIR=$cargo_target cargo build --release --offline --frozen --verbose
  cp $"($out)/cargo_target/release/($package)" $"($out)/bin/($package)"

  if ($cargo_home | path exists) {
    rm -r $cargo_home
  }
  rm -r $cargo_target
}
