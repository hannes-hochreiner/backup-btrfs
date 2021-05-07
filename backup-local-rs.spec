Name: backup-local-rs
Summary: Daemon to create local backups.
License: MIT
Version: 0.1.0
Release: 1%{?dist}
Source: %{name}-%{version}.tar
#BuildRequires: cargo

%description

%global debug_package %{nil}
%prep
%autosetup -n %{name}-%{version} -p1

%build
cargo build --release

%install
rm -rf $RPM_BUILD_ROOT
mkdir -p $RPM_BUILD_ROOT%{_bindir}
cp target/release/backup-local-rs $RPM_BUILD_ROOT%{_bindir}

%files
%{_bindir}/backup-local-rs