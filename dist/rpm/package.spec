Summary: %{_pkg_summary}
Name: %{_pkg_name}
Version: %{_pkg_version}
Release: %{_pkg_release}
License: %{_pkg_license}
Vendor: %{_pkg_vendor}
Group: %{_pkg_group}
URL: %{_pkg_url}

Source: master.tar.gz
Prefix: /usr

%description
%{_pkg_description}

%prep
%setup -n master

%build
echo ${RUSTUP_HOME}
echo ${CARGO_HOME}
echo ${HOME}
which cargo
cargo build --release

%install
cargo install --no-track --locked --root ${RPM_BUILD_ROOT}/usr --path .
rm ${RPM_BUILD_ROOT}/usr/.crates.toml

%files
/usr/bin/*

