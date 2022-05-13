#!/usr/bin/env bash

# sign
codesign \
  --sign="D5CABC53AFC47E57F4B688BB3688CF27830BAD36" \
  --identifier="com.techouse.alfred-gitmoji" \
  --deep \
  --force \
  --options=runtime \
  --entitlement="./entitlements.plist" \
  --timestamp \
  --verbose=4 \
  ./build/dist/gm

# verify
codesign -dv --verbose=4 ./build/dist/gm

# zip
zip -j ./build/dist/gm.zip ./build/dist/gm

# notarize
xcrun altool \
  --notarize-app \
  --primary-bundle-id="com.techouse.alfred-gitmoji" \
  --username="klemen.tusar@live.com" \
  --password="@keychain:Developer-altool" \
  --asc-provider="6LYC36B94Q" \
  --file="./build/dist/gm.zip"

# delete zip
rm ./build/dist/gm.zip
