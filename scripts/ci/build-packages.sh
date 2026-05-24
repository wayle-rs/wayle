#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
	echo "Usage: $0 <archive> <version>" >&2
	exit 1
fi

ARCHIVE="$(readlink -f "$1")"
VERSION="$2"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$PROJECT_DIR"

TOPDIR="${PWD}/rpmbuild"
mkdir -p "${TOPDIR}/SOURCES" "${TOPDIR}/SPECS"
cp "${ARCHIVE}" "${TOPDIR}/SOURCES/"
DATE="$(LC_ALL=C date '+%a %b %d %Y')"
sed \
	-e "s/@VERSION@/${VERSION}/g" \
	-e "s/@DATE@/${DATE}/g" \
	packaging/rpm/wayle.spec.in >"${TOPDIR}/SPECS/wayle.spec"
rpmbuild --define "_topdir ${TOPDIR}" --nodeps -bb "${TOPDIR}/SPECS/wayle.spec"
