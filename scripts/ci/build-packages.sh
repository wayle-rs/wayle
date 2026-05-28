#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 2 || $# -gt 3 ]]; then
	echo "Usage: $0 <archive> <version> [deb|rpm]" >&2
	exit 1
fi

ARCHIVE="$(readlink -f "$1")"
VERSION="$2"
DEB_FULLNAME="${DEBFULLNAME:-Jas Singh}"
DEB_EMAIL="${DEBEMAIL:-jaskiratpal.singh@outlook.com}"
MAINTAINER="${DEB_FULLNAME} <${DEB_EMAIL}>"
MAINTAINER_SED="$(printf '%s' "${MAINTAINER}" | sed 's/[&\\|]/\\&/g')"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$PROJECT_DIR"

render_template() {
	local template="$1"
	local output="$2"
	local date="$3"
	local date_sed
	local version_sed

	date_sed="$(printf '%s' "${date}" | sed 's/[&\\|]/\\&/g')"
	version_sed="$(printf '%s' "${VERSION}" | sed 's/[&\\|]/\\&/g')"
	sed \
		-e "s|@VERSION@|${version_sed}|g" \
		-e "s|@DATE@|${date_sed}|g" \
		-e "s|@MAINTAINER@|${MAINTAINER_SED}|g" \
		"${template}" >"${output}"
}

build_deb() {
	ARCHIVE_DIR="$(basename "$ARCHIVE" .tar.gz)"
	ARCHIVE_ARCH="${ARCHIVE_DIR%-linux}"
	ARCHIVE_ARCH="${ARCHIVE_ARCH##*-}"
	case "${ARCHIVE_ARCH}" in
	x86_64)
		DEB_ARCHITECTURE=amd64
		;;
	aarch64)
		DEB_ARCHITECTURE=arm64
		;;
	*)
		DEB_ARCHITECTURE="${ARCHIVE_ARCH}"
		;;
	esac
	TOPDIR="${PWD}/debbuild"
	BUILD_DIR="${TOPDIR}/BUILD"
	SOURCE_DIR="${BUILD_DIR}/source"
	PACKAGE_DIR="${SOURCE_DIR}/wayle-${VERSION}"
	OUT_DIR="${TOPDIR}/DEBS/${DEB_ARCHITECTURE}"
	rm -rf "${BUILD_DIR}"
	mkdir -p "${SOURCE_DIR}" "${OUT_DIR}"
	tar xzf "${ARCHIVE}" -C "${SOURCE_DIR}"
	mv "${SOURCE_DIR}/${ARCHIVE_DIR}" "${PACKAGE_DIR}"
	mkdir -p "${PACKAGE_DIR}/debian"
	mkdir -p "${PACKAGE_DIR}/debian/source"
	render_template packaging/debian/control.in "${PACKAGE_DIR}/debian/control" ""
	render_template packaging/debian/changelog.in "${PACKAGE_DIR}/debian/changelog" "$(LC_ALL=C date -R)"
	cp packaging/debian/copyright "${PACKAGE_DIR}/debian/copyright"
	cp packaging/debian/rules "${PACKAGE_DIR}/debian/rules"
	cp packaging/debian/source/format "${PACKAGE_DIR}/debian/source/format"

	(cd "${PACKAGE_DIR}" && dpkg-buildpackage -us -uc -b -a"${DEB_ARCHITECTURE}")
	find "${SOURCE_DIR}" -maxdepth 1 -type f -name '*.deb' -exec mv -t "${OUT_DIR}" {} +
}

build_rpm() {
	TOPDIR="${PWD}/rpmbuild"
	rm -rf "${TOPDIR}/BUILD" "${TOPDIR}/BUILDROOT"
	mkdir -p "${TOPDIR}/SOURCES" "${TOPDIR}/SPECS"
	cp "${ARCHIVE}" "${TOPDIR}/SOURCES/"
	DATE="$(LC_ALL=C date '+%a %b %d %Y')"
	render_template packaging/rpm/wayle.spec.in "${TOPDIR}/SPECS/wayle.spec" "${DATE}"
	rpmbuild --define "_topdir ${TOPDIR}" -bb "${TOPDIR}/SPECS/wayle.spec"
}

case "${3:-}" in
"")
	build_deb
	build_rpm
	;;
deb)
	build_deb
	;;
rpm)
	build_rpm
	;;
*)
	echo "Unknown package format: $3" >&2
	echo "Usage: $0 <archive> <version> [deb|rpm]" >&2
	exit 1
	;;
esac
