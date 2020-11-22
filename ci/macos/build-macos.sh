#!/bin/sh

OSTYPE=$(uname)

if [ "${OSTYPE}" != "Darwin" ]; then
    echo "[Error] macOS build script can be run on Darwin-type OS only."
    exit 1
fi

HAS_CMAKE=$(type cmake 2>/dev/null)

if [ "${HAS_CMAKE}" = "" ]; then
    echo "[Error] CMake not installed - please run 'install-dependencies-macos.sh' first."
    exit 1
fi

#export QT_PREFIX="$(find /usr/local/Cellar/qt5 -d 1 | tail -n 1)"
cp ../obs-studio/UI/obs-frontend-api/obs-frontend-api.h ../obs-studio/libobs
cp -r ../obs-studio/libobs ../obs-studio/obs

echo "=> Building plugin for macOS."
mkdir -p build && cd build
export QTDIR=/usr/local/opt/qt
export PATH=$PATH:$QTDIR/bin
QT_INCLUDE_DIR="$QTDIR/include" \
QT_LIB_DIR="$QTDIR/lib" \
LIBOBS_INCLUDE_DIR="$(pwd)/../../obs-studio" \
LIBOBS_LIB="$(pwd)/../../obs-studio/build/libobs" \
OBS_FRONTEND_LIB="$(pwd)/../../obs-studio/build/UI/obs-frontend-api" cargo build --release --features macos
