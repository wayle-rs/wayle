%global debug_package %{nil}

Name: wayle
Version: 0.2.1
Release: 1%{?dist}
Summary: A configurable desktop shell for Wayland compositors

License: MIT
URL: https://github.com/wayle-rs/wayle
Source0: {{{ git_dir_pack }}}

BuildRequires: pkgconfig(glib-2.0)
BuildRequires: pkgconfig(gdk-pixbuf-2.0)
BuildRequires: pkgconfig(pango)
BuildRequires: pkgconfig(cairo)
BuildRequires: cairo-gobject-devel
BuildRequires: pkgconfig(gtk4)
BuildRequires: pkgconfig(gtk4-layer-shell-0)
BuildRequires: pkgconfig(gtksourceview-5)
BuildRequires: pkgconfig(systemd)
BuildRequires: pkgconfig(xkbcommon)
BuildRequires: pkgconfig(libpulse)
BuildRequires: pkgconfig(fftw3)
BuildRequires: pkgconfig(libpipewire-0.3)
BuildRequires: clang-devel
BuildRequires: cargo
BuildRequires: pkgconf-pkg-config
BuildRequires: desktop-file-utils

Requires: gtk4
Requires: gtk4-layer-shell
Requires: pulseaudio-libs
Requires: fftw
Requires: pipewire-libs
Requires: libglvnd-gles
Requires: hicolor-icon-theme

Suggests: upower
Suggests: NetworkManager
Suggests: bluez
Suggests: power-profiles-daemon

%description
A configurable desktop shell for Wayland compositors. Built in Rust with GTK4 and Relm4. Compositor-agnostic successor to HyprPanel.

%prep
{{{ git_dir_setup_macro }}}
cargo fetch --target "$(rustc -vV | sed -n 's/host: //p')"

%build
cargo build --frozen --release

%install
install -Dm755 target/release/%{name} %{buildroot}%{_bindir}/%{name}
install -Dm755 target/release/%{name}-settings %{buildroot}%{_bindir}/%{name}-settings

install -d %{buildroot}%{_datadir}/icons/hicolor/scalable/actions
install -m 0644 resources/icons/hicolor/scalable/actions/*.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/actions/

target/release/%{name} completions bash > wayle.bash
target/release/%{name} completions zsh > _wayle
target/release/%{name} completions fish > wayle.fish

install -Dm644 wayle.bash %{buildroot}%{_datadir}/bash-completion/completions/wayle
install -Dm644 _wayle %{buildroot}%{_datadir}/zsh/site-functions/_wayle
install -Dm644 wayle.fish %{buildroot}%{_datadir}/fish/vendor_completions.d/wayle.fish

install -Dm644 resources/wayle.service %{buildroot}%{_userunitdir}/wayle.service
install -Dm644 resources/com.wayle.settings.desktop %{buildroot}%{_datadir}/applications/com.wayle.settings.desktop
desktop-file-validate %{buildroot}%{_datadir}/applications/com.wayle.settings.desktop
install -Dm644 resources/wayle-settings.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/wayle-settings.svg

%files
%{_bindir}/%{name}
%{_bindir}/%{name}-settings
%{_datadir}/icons/hicolor/scalable/actions/*.svg
%{_datadir}/bash-completion/completions/wayle
%{_datadir}/zsh/site-functions/_wayle
%{_datadir}/fish/vendor_completions.d/wayle.fish
%{_userunitdir}/%{name}.service
%{_datadir}/applications/com.wayle.settings.desktop
%{_datadir}/icons/hicolor/scalable/apps/wayle-settings.svg
%license LICENSE
%doc docs/
