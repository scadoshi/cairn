#!/usr/bin/env bash
# Patch the dx-generated MainActivity.kt so the Android OS back intent
# (edge-swipe gesture and hardware button) drives the Dioxus router instead of
# finishing the app. Ported from zwipe's zcripts/android/back_handler.sh.
#
# The router is in-memory, so the OS back never reaches it on its own; wry's
# Activity would finish the app. The unified OnBackPressedDispatcher catches
# both the gesture and the button and forwards it to the app as a `cairn:back`
# DOM event; the Rust side (components/navigation/back_handler.rs) decides:
# go_back, or finish the Activity at a root screen.
#
# dx regenerates MainActivity.kt on every `dx bundle`, so run this after
# `dx bundle` and before the Gradle repackage.
#
# Usage: scripts/android/back_handler.sh [MAIN_ACTIVITY_KT]
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
KT="${1:-$REPO_ROOT/target/dx/cairn/release/android/app/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt}"

[ -f "$KT" ] || { echo "MainActivity.kt not found: $KT" >&2; exit 1; }

cat > "$KT" <<'KT'
package dev.dioxus.main

import android.os.Bundle
import android.webkit.WebView
import androidx.activity.OnBackPressedCallback

typealias BuildConfig = com.scadoshi.count.BuildConfig

class MainActivity : WryActivity() {
    private var appWebView: WebView? = null

    override fun onWebViewCreate(webView: WebView) {
        super.onWebViewCreate(webView)
        appWebView = webView
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        // Intercept the OS back intent (gesture and button) and hand it to the
        // router by dispatching a DOM event the app listens for. The Rust side
        // owns the decision: navigate back, or finish to exit from a root.
        onBackPressedDispatcher.addCallback(this, object : OnBackPressedCallback(true) {
            override fun handleOnBackPressed() {
                appWebView?.evaluateJavascript(
                    "window.dispatchEvent(new Event('cairn:back'))",
                    null
                )
            }
        })
    }

    override fun onDestroy() {
        super.onDestroy()
        // The OS can destroy the Activity while keeping the process alive, and
        // the next onCreate re-runs wry's native init, which panics on
        // ndk-context's already-initialized assert (zwipe hit this in the
        // field). So the Activity always takes the process with it: every
        // reopen is a clean cold start.
        android.os.Process.killProcess(android.os.Process.myPid())
    }
}
KT

echo "Patched back-navigation into $KT"
