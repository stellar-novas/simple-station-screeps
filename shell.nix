let
  sources = import ./npins;
  pkgs = import sources.nixpkgs { };
in
#  fenix = import sources.fenix { };
pkgs.mkShell {
  # nativeBuildInputs is usually what you want -- tools you need to run
  LD_LIBRARY_PATH = "${pkgs.stdenv.cc.cc.lib}/lib";
  nativeBuildInputs = with pkgs.buildPackages; [
    nodejs
    stdenv.cc.cc
    #    fenix.complete.toolchain
  ];
}
