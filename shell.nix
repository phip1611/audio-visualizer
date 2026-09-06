{
  pkgs ? import <nixpkgs> { },
}:
let
  libs = with pkgs; [
    alsa-lib
    fontconfig
    libx11
    libxcursor
    libxkbcommon
  ];

in
pkgs.mkShell {
  packages =
    (with pkgs; [
      pkg-config
    ])
    ++ libs;

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath libs;
}
