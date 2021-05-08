# Generated by rust2rpm 17
%bcond_without check
%global __cargo_skip_build 0

%global crate backup-local-rs

Name:           rust-%{crate}
Version:        0.1.5
Release:        1%{?dist}
Summary:        Daemon to create local backups.

# Upstream license specification: None
License:        MIT

Source:         backup-local-rs-0.1.5.tar

#ExclusiveArch:  %{rust_arches}

BuildRequires:  glibc-langpack-en
BuildRequires:  rust-packaging
BuildRequires:  rust-anyhow+default-devel
BuildRequires:  rust-chrono+default-devel
BuildRequires:  rust-env_logger+default-devel
BuildRequires:  rust-log+default-devel
BuildRequires:  rust-serde+default-devel
BuildRequires:  rust-serde+derive-devel
BuildRequires:  rust-serde_json+default-devel
BuildRequires:  rust-uuid+default-devel

%global _description %{expand:
%{summary}.}

%description %{_description}

%package     -n %{crate}
Summary:        %{summary}

%description -n %{crate} %{_description}

%files       -n %{crate}
%{_bindir}/backup-local-rs

%prep
%autosetup -n %{crate}-%{version_no_tilde} -p1
%cargo_prep

%generate_buildrequires
%cargo_generate_buildrequires

%build
%cargo_build

%install
%cargo_install

%if %{with check}
%check
%cargo_test
%endif
