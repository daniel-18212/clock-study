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
manifest = original.replace(internet, internet + """
    <uses-permission android:name="android.permission.USE_EXACT_ALARM" />
    <uses-permission android:name="android.permission.SCHEDULE_EXACT_ALARM" android:maxSdkVersion="32" />""")
application_end = "    </application>"
if manifest.count(application_end) != 1:
    raise SystemExit("Android manifest template changed: could not add alarm receiver")
manifest = manifest.replace(application_end, """        <receiver android:name=".ClockAlarmReceiver" android:exported="false" />
    </application>""")
path.write_text(manifest, encoding="utf-8")
PY
fi

mkdir -p "$project/app/src/main/java/com/clockstudy/desktop"
cp packaging/android/ClockAlarmPlugin.kt "$project/app/src/main/java/com/clockstudy/desktop/"

rules="$project/app/proguard-rules.pro"
if ! grep -Fq 'com.clockstudy.desktop.ClockAlarmPlugin' "$rules"; then
  cat >> "$rules" <<'PROGUARD'

-keep class com.clockstudy.desktop.ClockAlarmPlugin { *; }
-keep class com.clockstudy.desktop.ClockAlarmArgs { *; }
PROGUARD
fi

icons="$(mktemp -d)"
trap 'rm -rf "$icons"' EXIT
npm run tauri icon -- src-tauri/icons/128x128.png --output "$icons" --ios-color '#000000'
cp -R "$icons/android/." "$project/app/src/main/res/"
