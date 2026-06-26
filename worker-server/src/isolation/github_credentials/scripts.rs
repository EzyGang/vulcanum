#[must_use]
pub(crate) fn git_config() -> &'static str {
    r#"[credential]
    helper =
    helper = "!f() { test \"$1\" = get || exit 0; protocol=; host=; while IFS= read -r line; do case \"$line\" in protocol=*) protocol=${line#protocol=} ;; host=*) host=${line#host=} ;; esac; done; if test \"$protocol\" = https && test \"$host\" = github.com && test -r \"$VULCANUM_GITHUB_TOKEN_FILE\"; then token=$(cat \"$VULCANUM_GITHUB_TOKEN_FILE\"); printf 'username=x-access-token\npassword=%s\n\n' \"$token\"; fi; }; f"
"#
}

#[must_use]
pub(crate) fn askpass_sh() -> &'static str {
    r#"#!/bin/sh
case "$1" in
    *Username*) printf '%s\n' 'x-access-token' ;;
    *)
        if [ -r "$VULCANUM_GITHUB_TOKEN_FILE" ]; then
            cat "$VULCANUM_GITHUB_TOKEN_FILE"
        fi
        ;;
esac
"#
}

#[must_use]
pub(crate) fn askpass_cmd() -> &'static str {
    r#"@echo off
set "VULCANUM_GITHUB_PROMPT=%~1"
if not "%VULCANUM_GITHUB_PROMPT:Username=%"=="%VULCANUM_GITHUB_PROMPT%" (
    echo x-access-token
    exit /b 0
)
if exist "%VULCANUM_GITHUB_TOKEN_FILE%" type "%VULCANUM_GITHUB_TOKEN_FILE%"
"#
}
