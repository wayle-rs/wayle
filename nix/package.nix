{
  cairo,
  fetchFromGitHub,
  fftw,
  glib,
  gtk4,
  gtk4-layer-shell,
  lib,
  libpulseaudio,
  libxkbcommon,
  llvmPackages,
  makeWrapper,
  pipewire,
  pkg-config,
  pulseaudio,
  rustPlatform,
  stdenv,
  wayland,
}:
rustPlatform.buildRustPackage (finalAttrs: rec {
  pname = "wayle";
  version = "0.1.0";

  # This gets overridden by the flake.
  src = fetchFromGitHub {
    owner = "Jas-SinghFSU";
    repo = "wayle";
    # tag = "v${finalAttrs.version}"; # Use this once the first tag is released.
    rev = "b4ade2c55c59f4a706192ff784ad11ba38517158"; # Once this ^, then delete this <.
    hash = "sha256-oc1EtpzH9KLKm0NUpqqQrfbYXftqjVvEfw72QbxlIfc=";
    fetchSubmodules = true;
  };

  cargoLock.lockFile = "${finalAttrs.src}/Cargo.lock";
  cargoHash = "sha256-Y/R8iI1sIHZxqxlPO1iTIqpLJGkCuTPAmh2OarvkoVA=";

  LIBCLANG_PATH = lib.makeLibraryPath [llvmPackages.libclang];
  LD_LIBRARY_PATH = lib.makeLibraryPath [
    cairo
    fftw
    glib
    gtk4
    gtk4-layer-shell
    libpulseaudio
    pipewire
    wayland
  ];

  buildInputs = [
    fftw
    gtk4
    gtk4-layer-shell
    libxkbcommon
    pipewire
    pulseaudio
  ];
  nativeBuildInputs = [
    gtk4
    makeWrapper
    pkg-config
  ];

  checkFlags = [
    # This test is broken because gtk fails to initialize in the nix sandbox.
    "--skip=css_loads_into_gtk4"

    # This test is broken because wayle is not given access to hyprland in the
    # nix sandbox.
    "--skip=new_fails_when_hyprland_instance_signature_missing"
  ];

  preBuild = ''
    # Cargo will fail without a home dir so give it a temporary one and we will
    # take out what we need from it later.
    export HOME=$TMPDIR

    # Add clang, glib, and other headers to bindgen search path.
    export BINDGEN_EXTRA_CLANG_ARGS="$(< ${stdenv.cc}/nix-support/libc-crt1-cflags) \
      $(< ${stdenv.cc}/nix-support/libc-cflags) \
      $(< ${stdenv.cc}/nix-support/cc-cflags) \
      $(< ${stdenv.cc}/nix-support/libcxx-cxxflags) \
      ${lib.optionalString stdenv.cc.isClang "-idirafter ${stdenv.cc.cc}/lib/clang/${lib.getVersion stdenv.cc.cc}/include"} \
      ${lib.optionalString stdenv.cc.isGNU "-isystem ${stdenv.cc.cc}/include/c++/${lib.getVersion stdenv.cc.cc} -isystem ${stdenv.cc.cc}/include/c++/${lib.getVersion stdenv.cc.cc}/${stdenv.hostPlatform.config} -idirafter ${stdenv.cc.cc}/lib/gcc/${stdenv.hostPlatform.config}/${lib.getVersion stdenv.cc.cc}/include"}"
  '';

  postInstall = ''
    # Generate wayle icons and copy them to the correct nix directory.
    mkdir -p $out/share
    $out/bin/wayle icons setup
    if [ -d "$TMPDIR/.local/share/wayle/icons" ]; then
      cp -r $TMPDIR/.local/share/wayle/icons $out/share/icons
    fi

    # Provide the correct library paths to the wayle binaries.
    wrapProgram "$out/bin/wayle" \
      --prefix LD_LIBRARY_PATH : "${LD_LIBRARY_PATH}"

    wrapProgram "$out/bin/wayle-shell" \
      --prefix LD_LIBRARY_PATH : "${LD_LIBRARY_PATH}"
  '';

  meta = {
    description = "A fast, configurable desktop environment shell for Wayland compositors";
    homepage = "https://github.com/Jas-SinghFSU/wayle";
    license = lib.licenses.mit;
    # maintainers = with lib.maintainers; []; # No maintainers yet :(
    mainProgram = "wayle";
  };
})
