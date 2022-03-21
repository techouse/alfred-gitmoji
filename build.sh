#!/usr/bin/env sh
if [ -d "build/dist" ]; then
  rm -rf build/dist
fi

if [ -d "build/debug_info" ]; then
  rm -rf build/debug_info
fi

mkdir -p build/dist build/debug_info
cp -r info.plist assets/* LICENSE README.md build/dist

if command -v dart-pubspec-licenses-lite; then
  dart-pubspec-licenses-lite --pubspec-lock pubspec.lock --output build/dist/OSS_LICENSES.txt
else
  echo 'Info: Unable to generate OSS_LICENSES.txt. Please install https://github.com/techouse/flutter_oss_licenses'
fi

dart compile exe bin/main.dart -o build/dist/gm -S build/debug_info/gm
