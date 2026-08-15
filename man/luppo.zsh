#compdef luppo

autoload -U is-at-least

_luppo() {
    typeset -A opt_args
    typeset -a _arguments_options
    local ret=1

    if is-at-least 5.2; then
        _arguments_options=(-s -S -C)
    else
        _arguments_options=(-s -C)
    fi

    local context curcontext="$curcontext" state line
    _arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
'-V[Print version]' \
'--version[Print version]' \
":: :_luppo_commands" \
"*::: :->luppo" \
&& ret=0
    case $state in
    (luppo)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:luppo-command-$line[1]:"
        case $line[1] in
            (add-repo)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
':name:_default' \
':url:_default' \
&& ret=0
;;
(blame)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
':package:_default' \
&& ret=0
;;
(build)
_arguments "${_arguments_options[@]}" : \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--target=[]:TARGET:_default' \
'--log-path=[]:LOG_PATH:_files' \
'--opt-level=[]:OPT_LEVEL:_default' \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'--no-sandbox[]' \
'--install-deps[]' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
'::spec:_default' \
&& ret=0
;;
(check-install)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
'*::packages:_default' \
&& ret=0
;;
(check-components)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
'::path:_default' \
&& ret=0
;;
(reset-history)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
'::path:_default' \
&& ret=0
;;
(check-repo)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'--circular[]' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(repo-diff)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
':index1:_default' \
':index2:_default' \
&& ret=0
;;
(toolchain)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'--start[]' \
'--update[]' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(clean)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(configure-pending)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(delete-cache)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(delta)
_arguments "${_arguments_options[@]}" : \
'--output-dir=[]:OUTPUT_DIR:_default' \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
'*::old_packages:_default' \
':new_package:_default' \
&& ret=0
;;
(disable-repo)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
':repo:_default' \
&& ret=0
;;
(emerge)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'--no-deps[]' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
'*::packages:_default' \
&& ret=0
;;
(emerge-up)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(enable-repo)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
':repo:_default' \
&& ret=0
;;
(fetch)
_arguments "${_arguments_options[@]}" : \
'-o+[]:OUTPUT_DIR:_default' \
'--output-dir=[]:OUTPUT_DIR:_default' \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'--runtime-deps[]' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
'*::packages:_default' \
&& ret=0
;;
(graph)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'--reverse[]' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
':package:_default' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
'::command:_default' \
&& ret=0
;;
(history)
_arguments "${_arguments_options[@]}" : \
'--from=[]:FROM:_default' \
'--to=[]:TO:_default' \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'--json[]' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(index)
_arguments "${_arguments_options[@]}" : \
'--output=[]:OUTPUT:_default' \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
':path:_default' \
&& ret=0
;;
(info)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
':package:_default' \
&& ret=0
;;
(install)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DESTDIR:_default' \
'--destdir=[]:DESTDIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'--reinstall[]' \
'--force[]' \
'--download-only[]' \
'--ignore-check[]' \
'--ignore-dependency[]' \
'--ignore-comar[]' \
'--ignore-file-conflict[]' \
'--ignore-package-conflict[]' \
'--ignore-safety[]' \
'--no-sandbox[]' \
'--install-deps[]' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'-h[Print help]' \
'--help[Print help]' \
'*::packages:_default' \
&& ret=0
;;
(list-available)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'--json[]' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(list-components)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'--json[]' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(list-files)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
':package:_default' \
&& ret=0
;;
(list-installed)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'--json[]' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(list-newest)
_arguments "${_arguments_options[@]}" : \
'--limit=[]:LIMIT:_default' \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(list-orphaned)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'--json[]' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(list-pending)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'--json[]' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(list-repo)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'--json[]' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(list-sources)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'--json[]' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(list-upgrades)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'--json[]' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(rebuild-db)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'--ignore-dependency[]' \
'--ignore-safety[]' \
'--ignore-comar[]' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
'*::packages:_default' \
&& ret=0
;;
(remove-orphaned)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'--json[]' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(remove-repo)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
':repo:_default' \
&& ret=0
;;
(rollback)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
':trace_id:_default' \
&& ret=0
;;
(search)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'--json[]' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
':query:_default' \
&& ret=0
;;
(search-file)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'--json[]' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
':path:_default' \
&& ret=0
;;
(update-repo)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'--json[]' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(upgrade)
_arguments "${_arguments_options[@]}" : \
'--component=[]:COMPONENT:_default' \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'--check-only[]' \
'--integrity-only[]' \
'--no-integrity[]' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
'*::packages:_default' \
&& ret=0
;;
(version)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(temp)
_arguments "${_arguments_options[@]}" : \
'-D+[]:DIR:_default' \
'--destdir=[]:DIR:_default' \
'-L+[]:KILOBYTES:_default' \
'--bandwidth-limit=[]:KILOBYTES:_default' \
'-u+[]:USERNAME:_default' \
'--username=[]:USERNAME:_default' \
'-p+[]:PASSWORD:_default' \
'--password=[]:PASSWORD:_default' \
'-j+[]:JOBS:_default' \
'--jobs=[]:JOBS:_default' \
'--log-path=[]:FILE:_files' \
'--opt-level=[]:LEVEL:_default' \
'-y[]' \
'--yes-all[]' \
'-v[]' \
'--verbose[]' \
'-d[]' \
'--debug[]' \
'-N[]' \
'--no-color[]' \
'--download-only[]' \
'--ignore-check[]' \
'-h[Print help]' \
'--help[Print help]' \
'::name:_default' \
&& ret=0
;;
        esac
    ;;
esac
}

(( $+functions[_luppo_commands] )) ||
_luppo_commands() {
    local commands; commands=(
'add-repo:' \
'blame:' \
'build:' \
'check-install:' \
'check-components:' \
'reset-history:' \
'check-repo:' \
'repo-diff:' \
'toolchain:' \
'clean:' \
'configure-pending:' \
'delete-cache:' \
'delta:' \
'disable-repo:' \
'emerge:' \
'emerge-up:' \
'enable-repo:' \
'fetch:' \
'graph:' \
'help:' \
'history:' \
'index:' \
'info:' \
'install:' \
'list-available:' \
'list-components:' \
'list-files:' \
'list-installed:' \
'list-newest:' \
'list-orphaned:' \
'list-pending:' \
'list-repo:' \
'list-sources:' \
'list-upgrades:' \
'rebuild-db:' \
'remove:' \
'remove-orphaned:' \
'remove-repo:' \
'rollback:' \
'search:' \
'search-file:' \
'update-repo:' \
'upgrade:' \
'version:' \
'temp:' \
    )
    _describe -t commands 'luppo commands' commands "$@"
}
(( $+functions[_luppo__subcmd__add-repo_commands] )) ||
_luppo__subcmd__add-repo_commands() {
    local commands; commands=()
    _describe -t commands 'luppo add-repo commands' commands "$@"
}
(( $+functions[_luppo__subcmd__blame_commands] )) ||
_luppo__subcmd__blame_commands() {
    local commands; commands=()
    _describe -t commands 'luppo blame commands' commands "$@"
}
(( $+functions[_luppo__subcmd__build_commands] )) ||
_luppo__subcmd__build_commands() {
    local commands; commands=()
    _describe -t commands 'luppo build commands' commands "$@"
}
(( $+functions[_luppo__subcmd__check-components_commands] )) ||
_luppo__subcmd__check-components_commands() {
    local commands; commands=()
    _describe -t commands 'luppo check-components commands' commands "$@"
}
(( $+functions[_luppo__subcmd__check-install_commands] )) ||
_luppo__subcmd__check-install_commands() {
    local commands; commands=()
    _describe -t commands 'luppo check-install commands' commands "$@"
}
(( $+functions[_luppo__subcmd__check-repo_commands] )) ||
_luppo__subcmd__check-repo_commands() {
    local commands; commands=()
    _describe -t commands 'luppo check-repo commands' commands "$@"
}
(( $+functions[_luppo__subcmd__clean_commands] )) ||
_luppo__subcmd__clean_commands() {
    local commands; commands=()
    _describe -t commands 'luppo clean commands' commands "$@"
}
(( $+functions[_luppo__subcmd__configure-pending_commands] )) ||
_luppo__subcmd__configure-pending_commands() {
    local commands; commands=()
    _describe -t commands 'luppo configure-pending commands' commands "$@"
}
(( $+functions[_luppo__subcmd__delete-cache_commands] )) ||
_luppo__subcmd__delete-cache_commands() {
    local commands; commands=()
    _describe -t commands 'luppo delete-cache commands' commands "$@"
}
(( $+functions[_luppo__subcmd__delta_commands] )) ||
_luppo__subcmd__delta_commands() {
    local commands; commands=()
    _describe -t commands 'luppo delta commands' commands "$@"
}
(( $+functions[_luppo__subcmd__disable-repo_commands] )) ||
_luppo__subcmd__disable-repo_commands() {
    local commands; commands=()
    _describe -t commands 'luppo disable-repo commands' commands "$@"
}
(( $+functions[_luppo__subcmd__emerge_commands] )) ||
_luppo__subcmd__emerge_commands() {
    local commands; commands=()
    _describe -t commands 'luppo emerge commands' commands "$@"
}
(( $+functions[_luppo__subcmd__emerge-up_commands] )) ||
_luppo__subcmd__emerge-up_commands() {
    local commands; commands=()
    _describe -t commands 'luppo emerge-up commands' commands "$@"
}
(( $+functions[_luppo__subcmd__enable-repo_commands] )) ||
_luppo__subcmd__enable-repo_commands() {
    local commands; commands=()
    _describe -t commands 'luppo enable-repo commands' commands "$@"
}
(( $+functions[_luppo__subcmd__fetch_commands] )) ||
_luppo__subcmd__fetch_commands() {
    local commands; commands=()
    _describe -t commands 'luppo fetch commands' commands "$@"
}
(( $+functions[_luppo__subcmd__graph_commands] )) ||
_luppo__subcmd__graph_commands() {
    local commands; commands=()
    _describe -t commands 'luppo graph commands' commands "$@"
}
(( $+functions[_luppo__subcmd__help_commands] )) ||
_luppo__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'luppo help commands' commands "$@"
}
(( $+functions[_luppo__subcmd__history_commands] )) ||
_luppo__subcmd__history_commands() {
    local commands; commands=()
    _describe -t commands 'luppo history commands' commands "$@"
}
(( $+functions[_luppo__subcmd__index_commands] )) ||
_luppo__subcmd__index_commands() {
    local commands; commands=()
    _describe -t commands 'luppo index commands' commands "$@"
}
(( $+functions[_luppo__subcmd__info_commands] )) ||
_luppo__subcmd__info_commands() {
    local commands; commands=()
    _describe -t commands 'luppo info commands' commands "$@"
}
(( $+functions[_luppo__subcmd__install_commands] )) ||
_luppo__subcmd__install_commands() {
    local commands; commands=()
    _describe -t commands 'luppo install commands' commands "$@"
}
(( $+functions[_luppo__subcmd__list-available_commands] )) ||
_luppo__subcmd__list-available_commands() {
    local commands; commands=()
    _describe -t commands 'luppo list-available commands' commands "$@"
}
(( $+functions[_luppo__subcmd__list-components_commands] )) ||
_luppo__subcmd__list-components_commands() {
    local commands; commands=()
    _describe -t commands 'luppo list-components commands' commands "$@"
}
(( $+functions[_luppo__subcmd__list-files_commands] )) ||
_luppo__subcmd__list-files_commands() {
    local commands; commands=()
    _describe -t commands 'luppo list-files commands' commands "$@"
}
(( $+functions[_luppo__subcmd__list-installed_commands] )) ||
_luppo__subcmd__list-installed_commands() {
    local commands; commands=()
    _describe -t commands 'luppo list-installed commands' commands "$@"
}
(( $+functions[_luppo__subcmd__list-newest_commands] )) ||
_luppo__subcmd__list-newest_commands() {
    local commands; commands=()
    _describe -t commands 'luppo list-newest commands' commands "$@"
}
(( $+functions[_luppo__subcmd__list-orphaned_commands] )) ||
_luppo__subcmd__list-orphaned_commands() {
    local commands; commands=()
    _describe -t commands 'luppo list-orphaned commands' commands "$@"
}
(( $+functions[_luppo__subcmd__list-pending_commands] )) ||
_luppo__subcmd__list-pending_commands() {
    local commands; commands=()
    _describe -t commands 'luppo list-pending commands' commands "$@"
}
(( $+functions[_luppo__subcmd__list-repo_commands] )) ||
_luppo__subcmd__list-repo_commands() {
    local commands; commands=()
    _describe -t commands 'luppo list-repo commands' commands "$@"
}
(( $+functions[_luppo__subcmd__list-sources_commands] )) ||
_luppo__subcmd__list-sources_commands() {
    local commands; commands=()
    _describe -t commands 'luppo list-sources commands' commands "$@"
}
(( $+functions[_luppo__subcmd__list-upgrades_commands] )) ||
_luppo__subcmd__list-upgrades_commands() {
    local commands; commands=()
    _describe -t commands 'luppo list-upgrades commands' commands "$@"
}
(( $+functions[_luppo__subcmd__rebuild-db_commands] )) ||
_luppo__subcmd__rebuild-db_commands() {
    local commands; commands=()
    _describe -t commands 'luppo rebuild-db commands' commands "$@"
}
(( $+functions[_luppo__subcmd__remove_commands] )) ||
_luppo__subcmd__remove_commands() {
    local commands; commands=()
    _describe -t commands 'luppo remove commands' commands "$@"
}
(( $+functions[_luppo__subcmd__remove-orphaned_commands] )) ||
_luppo__subcmd__remove-orphaned_commands() {
    local commands; commands=()
    _describe -t commands 'luppo remove-orphaned commands' commands "$@"
}
(( $+functions[_luppo__subcmd__remove-repo_commands] )) ||
_luppo__subcmd__remove-repo_commands() {
    local commands; commands=()
    _describe -t commands 'luppo remove-repo commands' commands "$@"
}
(( $+functions[_luppo__subcmd__repo-diff_commands] )) ||
_luppo__subcmd__repo-diff_commands() {
    local commands; commands=()
    _describe -t commands 'luppo repo-diff commands' commands "$@"
}
(( $+functions[_luppo__subcmd__reset-history_commands] )) ||
_luppo__subcmd__reset-history_commands() {
    local commands; commands=()
    _describe -t commands 'luppo reset-history commands' commands "$@"
}
(( $+functions[_luppo__subcmd__rollback_commands] )) ||
_luppo__subcmd__rollback_commands() {
    local commands; commands=()
    _describe -t commands 'luppo rollback commands' commands "$@"
}
(( $+functions[_luppo__subcmd__search_commands] )) ||
_luppo__subcmd__search_commands() {
    local commands; commands=()
    _describe -t commands 'luppo search commands' commands "$@"
}
(( $+functions[_luppo__subcmd__search-file_commands] )) ||
_luppo__subcmd__search-file_commands() {
    local commands; commands=()
    _describe -t commands 'luppo search-file commands' commands "$@"
}
(( $+functions[_luppo__subcmd__temp_commands] )) ||
_luppo__subcmd__temp_commands() {
    local commands; commands=()
    _describe -t commands 'luppo temp commands' commands "$@"
}
(( $+functions[_luppo__subcmd__toolchain_commands] )) ||
_luppo__subcmd__toolchain_commands() {
    local commands; commands=()
    _describe -t commands 'luppo toolchain commands' commands "$@"
}
(( $+functions[_luppo__subcmd__update-repo_commands] )) ||
_luppo__subcmd__update-repo_commands() {
    local commands; commands=()
    _describe -t commands 'luppo update-repo commands' commands "$@"
}
(( $+functions[_luppo__subcmd__upgrade_commands] )) ||
_luppo__subcmd__upgrade_commands() {
    local commands; commands=()
    _describe -t commands 'luppo upgrade commands' commands "$@"
}
(( $+functions[_luppo__subcmd__version_commands] )) ||
_luppo__subcmd__version_commands() {
    local commands; commands=()
    _describe -t commands 'luppo version commands' commands "$@"
}

if [ "$funcstack[1]" = "_luppo" ]; then
    _luppo "$@"
else
    compdef _luppo luppo
fi
