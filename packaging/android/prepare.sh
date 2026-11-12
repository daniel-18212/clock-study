#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/../.."
project=src-tauri/gen/android
if [[ ! -f "$project/gradlew" ]]; then
  npm run tauri android init -- --ci --skip-targets-install
fi

# Exact alarms are part of the timer's core function. Android 13+ grants
# USE_EXACT_ALARM on install; Android 12 uses the older permission.
manifest="$project/app/src/main/AndroidManifest.xml"
if ! grep -Fq 'android.permission.USE_EXACT_ALARM' "$manifest"; then
  python3 - "$manifest" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
original = path.read_text(encoding="utf-8")
internet = '    <uses-permission android:name="android.permission.INTERNET" />'
if original.count(internet) != 1:
    raise SystemExit("Android manifest template changed: could not add alarm permissions")
path.write_text(original.replace(internet, internet + """
    <uses-permission android:name="android.permission.USE_EXACT_ALARM" />
    <uses-permission android:name="android.permission.SCHEDULE_EXACT_ALARM" android:maxSdkVersion="32" />"""), encoding="utf-8")
PY
fi

icons="$(mktemp -d)"
trap 'rm -rf "$icons"' EXIT
npm run tauri icon -- src-tauri/icons/128x128.png --output "$icons" --ios-color '#000000'
cp -R "$icons/android/." "$project/app/src/main/res/"
