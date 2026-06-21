Name:           wayle
Version:        0.6.0
Release:        1%{?dist}
Summary:        Wayland Elements - A compositor agnostic desktop shell
License:        MIT
URL:            https://wayle.app/
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  curl
BuildRequires:  gcc
BuildRequires:  clang
BuildRequires:  cmake
BuildRequires:  git
BuildRequires:  pkgconf-pkg-config
BuildRequires:  gtk4-devel
BuildRequires:  gtk4-layer-shell-devel
BuildRequires:  gtksourceview5-devel
BuildRequires:  libxkbcommon-devel
BuildRequires:  pipewire-devel
BuildRequires:  pulseaudio-libs-devel
BuildRequires:  fftw-devel
BuildRequires:  systemd-devel
BuildRequires:  desktop-file-utils

ExclusiveArch:  x86_64 aarch64

Requires:       hicolor-icon-theme
Suggests:       pipewire-pulseaudio
Suggests:       wireplumber
Suggests:       NetworkManager
Suggests:       bluez
Suggests:       upower
Suggests:       power-profiles-daemon

%description
Wayle is a Wayland desktop shell with the bar, notifications, OSD, wallpaper,
and device controls built in. Written in Rust with GTK4 and Relm4.

%prep
%autosetup -n %{name}-%{version}

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
export PATH="$HOME/.cargo/bin:$PATH"
rustc --version
cargo --version

%build
export PATH="$HOME/.cargo/bin:$PATH"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-1}"
export RUSTFLAGS="$RUSTFLAGS -C lto=no -C codegen-units=8"
cargo build --workspace --release

%install
export PATH="$HOME/.cargo/bin:$PATH"
install -Dm0755 target/release/wayle %{buildroot}%{_bindir}/wayle
install -Dm0755 target/release/wayle-settings %{buildroot}%{_bindir}/wayle-settings

install -Dm0644 resources/wayle.service %{buildroot}%{_userunitdir}/wayle.service
install -Dm0644 resources/com.wayle.settings.desktop %{buildroot}%{_datadir}/applications/com.wayle.settings.desktop
desktop-file-validate %{buildroot}%{_datadir}/applications/com.wayle.settings.desktop

install -Dm0644 resources/wayle-settings.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/wayle-settings.svg

install -d %{buildroot}%{_datadir}/icons/hicolor/scalable/actions
install -m0644 resources/icons/hicolor/scalable/actions/*.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/actions/

%files
%license LICENSE
%doc README.md
%{_bindir}/wayle
%{_bindir}/wayle-settings
%{_userunitdir}/wayle.service
%{_datadir}/applications/com.wayle.settings.desktop
%{_datadir}/icons/hicolor/scalable/apps/wayle-settings.svg
%{_datadir}/icons/hicolor/scalable/actions/*.svg

%changelog
* Sat Jun 21 2026 killcrb - 0.6.0-1
- Initial COPR package
