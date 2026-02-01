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

  meta.mainProgram = "fw16-kblight-daemon";
}
