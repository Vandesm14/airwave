{pkgs}: {
  deps = [
    pkgs.pkg-config
    pkgs.openssl
    pkgs.nodePackages.pnpm
    pkgs.mprocs
  ];
}
