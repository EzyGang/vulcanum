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

#[must_use]
pub(crate) fn gh_wrapper_sh() -> &'static str {
    r#"#!/bin/sh
wrapper_path=${VULCANUM_GITHUB_GH_WRAPPER:-}
real_gh=
old_ifs=$IFS
IFS=:
for dir in $PATH; do
    if [ -z "$dir" ]; then
        dir=.
    fi
    candidate="$dir/gh"
    if [ "$candidate" != "$wrapper_path" ] && [ -x "$candidate" ]; then
        real_gh=$candidate
        break
    fi
done
IFS=$old_ifs

if [ -z "$real_gh" ]; then
    printf '%s\n' 'vulcanum gh wrapper: real gh not found' >&2
    exit 127
fi

if [ -r "$VULCANUM_GITHUB_TOKEN_FILE" ]; then
    token=$(cat "$VULCANUM_GITHUB_TOKEN_FILE")
    export GH_TOKEN="$token"
    export GITHUB_TOKEN="$token"
fi

exec "$real_gh" "$@"
"#
}

#[must_use]
pub(crate) fn gh_wrapper_cmd() -> &'static str {
    r#"@echo off
setlocal EnableExtensions DisableDelayedExpansion
set "VULCANUM_REAL_GH="
for %%N in (gh.exe gh.cmd gh.bat gh) do (
    for /f "delims=" %%P in ('where %%N 2^>nul') do (
        if /I not "%%~fP"=="%VULCANUM_GITHUB_GH_WRAPPER%" if /I not "%%~fP"=="%~f0" (
            set "VULCANUM_REAL_GH=%%~fP"
            goto vulcanum_found_gh
        )
    )
)
:vulcanum_found_gh
if not defined VULCANUM_REAL_GH (
    >&2 echo vulcanum gh wrapper: real gh not found
    exit /b 127
)
if exist "%VULCANUM_GITHUB_TOKEN_FILE%" (
    set /p VULCANUM_GITHUB_TOKEN=<"%VULCANUM_GITHUB_TOKEN_FILE%"
    set "GH_TOKEN=%VULCANUM_GITHUB_TOKEN%"
    set "GITHUB_TOKEN=%VULCANUM_GITHUB_TOKEN%"
)
"%VULCANUM_REAL_GH%" %*
exit /b %ERRORLEVEL%
"#
}
