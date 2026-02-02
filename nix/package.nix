{
  craneLib,
  udev,
  pkg-config,
}:
craneLib.buildPackage {
  src = ./..;
  strictDeps = true;

  buildInputs = [udev];
  nativeBuildInputs = [pkg-config];

  postInstall = ''
    mkdir -p $out/lib/systemd/system
    substituteAll fw16-kblight-daemon.service $out/lib/systemd/system/fw16-kblight-daemon.service
  '';

  meta.mainProgram = "fw16-kblight-daemon";
}
