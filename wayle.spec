%global debug_package %{nil}

Name: wayle
Version: 0.2.0
Release: 1%{dist}
Summary: A configurable desktop shell for Wayland compositors

License: MIT
URL: https://github.com/wayle-rs/wayle
Source: %{url}/archive/v%{version}.tar.gz

BuildRequires: glib2-devel
BuildRequires: gdk-pixbuf2-devel
BuildRequires: pango-devel
BuildRequires: cairo-devel
BuildRequires: cairo-gobject-devel
BuildRequires: gtk4-devel
BuildRequires: gtk4-layer-shell-devel
BuildRequires: systemd-devel
BuildRequires: libxkbcommon-devel
BuildRequires: pulseaudio-libs-devel
BuildRequires: fftw-devel
BuildRequires: pipewire-devel
BuildRequires: clang-devel
BuildRequires: cargo
BuildRequires: pkgconf-pkg-config

Requires: gtk4
Requires: gtk4-layer-shell
Requires: pulseaudio-libs
Requires: fftw
Requires: pipewire-libs

Suggests: upower
Suggests: NetworkManager
Suggests: bluez
Suggests: power-profiles-daemon

%description
A configurable desktop shell for Wayland compositors. Built in Rust with GTK4 and Relm4. Compositor-agnostic successor to HyprPanel.

%prep
%autosetup -n %{name}-%{version}
cargo fetch --target "$(rustc -vV | sed -n 's/host: //p')"

%build
cargo build --frozen --release

%install
install -Dm755 target/release/%{name} %{buildroot}%{_bindir}/%{name} 

install -dm755 %{buildroot}%{_datadir}/%{name}/icons
cp -r resources/icons/hicolor %{buildroot}%{_datadir}/%{name}/icons

target/release/%{name} completions bash > wayle.bash
target/release/%{name} completions zsh > _wayle
target/release/%{name} completions fish > wayle.fish

install -Dm644 wayle.bash %{buildroot}%{_datadir}/bash-completion/completions/wayle
install -Dm644 _wayle %{buildroot}%{_datadir}/zsh/site-functions/_wayle
install -Dm644 wayle.fish %{buildroot}%{_datadir}/fish/vendor_completions.d/wayle.fish

install -Dm644 resources/%{name}.service %{buildroot}%{_userunitdir}/%{name}.service

%files
%{_bindir}/%{name}
%{_datadir}/%{name}/
%{_datadir}/bash-completion/completions/wayle
%{_datadir}/zsh/site-functions/_wayle
%{_datadir}/fish/vendor_completions.d/wayle.fish
%{_userunitdir}/%{name}.service
%license LICENSE
%doc docs/
