{
  pkgs ? import <nixpkgs> { },
}:
let
  libs = with pkgs; [
    alsa-lib
    libGL
    libx11
    libxcursor
    libxi
    libxkbcommon
    libxrandr
    wayland
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
