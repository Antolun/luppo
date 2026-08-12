# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_pisi_global_optspecs
	string join \n D/destdir= y/yes-all v/verbose d/debug N/no-color L/bandwidth-limit= u/username= p/password= j/jobs= download-only ignore-check log-path= opt-level= h/help V/version
end

function __fish_pisi_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_pisi_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_pisi_using_subcommand
	set -l cmd (__fish_pisi_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c pisi -n "__fish_pisi_needs_command" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_needs_command" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_needs_command" -s u -l username -r
complete -c pisi -n "__fish_pisi_needs_command" -s p -l password -r
complete -c pisi -n "__fish_pisi_needs_command" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_needs_command" -l log-path -r -F
complete -c pisi -n "__fish_pisi_needs_command" -l opt-level -r
complete -c pisi -n "__fish_pisi_needs_command" -s y -l yes-all
complete -c pisi -n "__fish_pisi_needs_command" -s v -l verbose
complete -c pisi -n "__fish_pisi_needs_command" -s d -l debug
complete -c pisi -n "__fish_pisi_needs_command" -s N -l no-color
complete -c pisi -n "__fish_pisi_needs_command" -l download-only
complete -c pisi -n "__fish_pisi_needs_command" -l ignore-check
complete -c pisi -n "__fish_pisi_needs_command" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_needs_command" -s V -l version -d 'Print version'
complete -c pisi -n "__fish_pisi_needs_command" -f -a "add-repo"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "blame"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "build"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "check-install"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "check-components"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "reset-history"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "check-repo"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "repo-diff"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "toolchain"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "clean"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "configure-pending"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "delete-cache"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "delta"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "disable-repo"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "emerge"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "emerge-up"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "enable-repo"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "fetch"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "graph"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "help"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "history"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "index"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "info"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "install"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "list-available"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "list-components"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "list-files"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "list-installed"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "list-newest"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "list-orphaned"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "list-pending"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "list-repo"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "list-sources"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "list-upgrades"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "rebuild-db"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "remove"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "remove-orphaned"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "remove-repo"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "rollback"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "search"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "search-file"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "update-repo"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "upgrade"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "version"
complete -c pisi -n "__fish_pisi_needs_command" -f -a "temp"
complete -c pisi -n "__fish_pisi_using_subcommand add-repo" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand add-repo" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand add-repo" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand add-repo" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand add-repo" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand add-repo" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand add-repo" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand add-repo" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand add-repo" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand add-repo" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand add-repo" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand add-repo" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand add-repo" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand add-repo" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand blame" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand blame" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand blame" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand blame" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand blame" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand blame" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand blame" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand blame" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand blame" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand blame" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand blame" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand blame" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand blame" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand blame" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand build" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand build" -l target -r
complete -c pisi -n "__fish_pisi_using_subcommand build" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand build" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand build" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand build" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand build" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand build" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand build" -l no-sandbox
complete -c pisi -n "__fish_pisi_using_subcommand build" -l install-deps
complete -c pisi -n "__fish_pisi_using_subcommand build" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand build" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand build" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand build" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand build" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand build" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand build" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand check-install" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand check-install" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand check-install" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand check-install" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand check-install" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand check-install" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand check-install" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand check-install" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand check-install" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand check-install" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand check-install" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand check-install" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand check-install" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand check-install" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand check-components" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand check-components" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand check-components" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand check-components" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand check-components" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand check-components" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand check-components" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand check-components" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand check-components" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand check-components" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand check-components" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand check-components" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand check-components" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand check-components" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand reset-history" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand reset-history" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand reset-history" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand reset-history" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand reset-history" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand reset-history" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand reset-history" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand reset-history" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand reset-history" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand reset-history" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand reset-history" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand reset-history" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand reset-history" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand reset-history" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand check-repo" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand check-repo" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand check-repo" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand check-repo" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand check-repo" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand check-repo" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand check-repo" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand check-repo" -l circular
complete -c pisi -n "__fish_pisi_using_subcommand check-repo" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand check-repo" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand check-repo" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand check-repo" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand check-repo" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand check-repo" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand check-repo" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand repo-diff" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand repo-diff" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand repo-diff" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand repo-diff" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand repo-diff" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand repo-diff" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand repo-diff" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand repo-diff" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand repo-diff" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand repo-diff" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand repo-diff" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand repo-diff" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand repo-diff" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand repo-diff" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand toolchain" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand toolchain" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand toolchain" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand toolchain" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand toolchain" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand toolchain" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand toolchain" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand toolchain" -l start
complete -c pisi -n "__fish_pisi_using_subcommand toolchain" -l update
complete -c pisi -n "__fish_pisi_using_subcommand toolchain" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand toolchain" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand toolchain" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand toolchain" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand toolchain" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand toolchain" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand toolchain" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand clean" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand clean" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand clean" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand clean" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand clean" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand clean" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand clean" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand clean" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand clean" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand clean" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand clean" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand clean" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand clean" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand clean" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand configure-pending" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand configure-pending" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand configure-pending" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand configure-pending" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand configure-pending" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand configure-pending" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand configure-pending" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand configure-pending" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand configure-pending" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand configure-pending" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand configure-pending" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand configure-pending" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand configure-pending" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand configure-pending" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand delete-cache" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand delete-cache" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand delete-cache" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand delete-cache" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand delete-cache" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand delete-cache" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand delete-cache" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand delete-cache" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand delete-cache" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand delete-cache" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand delete-cache" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand delete-cache" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand delete-cache" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand delete-cache" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand delta" -l output-dir -r
complete -c pisi -n "__fish_pisi_using_subcommand delta" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand delta" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand delta" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand delta" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand delta" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand delta" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand delta" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand delta" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand delta" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand delta" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand delta" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand delta" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand delta" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand delta" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand disable-repo" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand disable-repo" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand disable-repo" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand disable-repo" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand disable-repo" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand disable-repo" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand disable-repo" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand disable-repo" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand disable-repo" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand disable-repo" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand disable-repo" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand disable-repo" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand disable-repo" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand disable-repo" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand emerge" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand emerge" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand emerge" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand emerge" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand emerge" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand emerge" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand emerge" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand emerge" -l no-deps
complete -c pisi -n "__fish_pisi_using_subcommand emerge" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand emerge" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand emerge" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand emerge" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand emerge" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand emerge" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand emerge" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand emerge-up" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand emerge-up" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand emerge-up" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand emerge-up" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand emerge-up" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand emerge-up" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand emerge-up" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand emerge-up" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand emerge-up" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand emerge-up" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand emerge-up" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand emerge-up" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand emerge-up" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand emerge-up" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand enable-repo" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand enable-repo" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand enable-repo" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand enable-repo" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand enable-repo" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand enable-repo" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand enable-repo" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand enable-repo" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand enable-repo" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand enable-repo" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand enable-repo" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand enable-repo" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand enable-repo" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand enable-repo" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand fetch" -s o -l output-dir -r
complete -c pisi -n "__fish_pisi_using_subcommand fetch" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand fetch" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand fetch" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand fetch" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand fetch" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand fetch" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand fetch" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand fetch" -l runtime-deps
complete -c pisi -n "__fish_pisi_using_subcommand fetch" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand fetch" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand fetch" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand fetch" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand fetch" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand fetch" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand fetch" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand graph" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand graph" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand graph" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand graph" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand graph" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand graph" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand graph" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand graph" -l reverse
complete -c pisi -n "__fish_pisi_using_subcommand graph" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand graph" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand graph" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand graph" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand graph" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand graph" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand graph" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand help" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand help" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand help" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand help" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand help" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand help" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand help" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand help" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand help" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand help" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand help" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand help" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand help" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand help" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand history" -l from -r
complete -c pisi -n "__fish_pisi_using_subcommand history" -l to -r
complete -c pisi -n "__fish_pisi_using_subcommand history" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand history" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand history" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand history" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand history" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand history" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand history" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand history" -l json
complete -c pisi -n "__fish_pisi_using_subcommand history" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand history" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand history" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand history" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand history" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand history" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand history" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand index" -l output -r
complete -c pisi -n "__fish_pisi_using_subcommand index" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand index" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand index" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand index" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand index" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand index" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand index" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand index" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand index" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand index" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand index" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand index" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand index" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand index" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand info" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand info" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand info" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand info" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand info" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand info" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand info" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand info" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand info" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand info" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand info" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand info" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand info" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand info" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand install" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand install" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand install" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand install" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand install" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand install" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand install" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand install" -l reinstall
complete -c pisi -n "__fish_pisi_using_subcommand install" -l force
complete -c pisi -n "__fish_pisi_using_subcommand install" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand install" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand install" -l ignore-dependency
complete -c pisi -n "__fish_pisi_using_subcommand install" -l ignore-comar
complete -c pisi -n "__fish_pisi_using_subcommand install" -l ignore-file-conflict
complete -c pisi -n "__fish_pisi_using_subcommand install" -l ignore-package-conflict
complete -c pisi -n "__fish_pisi_using_subcommand install" -l ignore-safety
complete -c pisi -n "__fish_pisi_using_subcommand install" -l no-sandbox
complete -c pisi -n "__fish_pisi_using_subcommand install" -l install-deps
complete -c pisi -n "__fish_pisi_using_subcommand install" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand install" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand install" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand install" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand install" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand list-available" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand list-available" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand list-available" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand list-available" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand list-available" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand list-available" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand list-available" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand list-available" -l json
complete -c pisi -n "__fish_pisi_using_subcommand list-available" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand list-available" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand list-available" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand list-available" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand list-available" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand list-available" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand list-available" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand list-components" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand list-components" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand list-components" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand list-components" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand list-components" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand list-components" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand list-components" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand list-components" -l json
complete -c pisi -n "__fish_pisi_using_subcommand list-components" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand list-components" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand list-components" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand list-components" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand list-components" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand list-components" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand list-components" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand list-files" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand list-files" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand list-files" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand list-files" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand list-files" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand list-files" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand list-files" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand list-files" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand list-files" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand list-files" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand list-files" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand list-files" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand list-files" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand list-files" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand list-installed" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand list-installed" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand list-installed" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand list-installed" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand list-installed" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand list-installed" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand list-installed" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand list-installed" -l json
complete -c pisi -n "__fish_pisi_using_subcommand list-installed" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand list-installed" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand list-installed" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand list-installed" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand list-installed" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand list-installed" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand list-installed" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand list-newest" -l limit -r
complete -c pisi -n "__fish_pisi_using_subcommand list-newest" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand list-newest" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand list-newest" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand list-newest" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand list-newest" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand list-newest" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand list-newest" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand list-newest" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand list-newest" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand list-newest" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand list-newest" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand list-newest" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand list-newest" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand list-newest" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand list-orphaned" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand list-orphaned" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand list-orphaned" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand list-orphaned" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand list-orphaned" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand list-orphaned" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand list-orphaned" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand list-orphaned" -l json
complete -c pisi -n "__fish_pisi_using_subcommand list-orphaned" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand list-orphaned" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand list-orphaned" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand list-orphaned" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand list-orphaned" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand list-orphaned" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand list-orphaned" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand list-pending" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand list-pending" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand list-pending" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand list-pending" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand list-pending" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand list-pending" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand list-pending" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand list-pending" -l json
complete -c pisi -n "__fish_pisi_using_subcommand list-pending" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand list-pending" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand list-pending" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand list-pending" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand list-pending" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand list-pending" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand list-pending" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand list-repo" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand list-repo" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand list-repo" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand list-repo" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand list-repo" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand list-repo" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand list-repo" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand list-repo" -l json
complete -c pisi -n "__fish_pisi_using_subcommand list-repo" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand list-repo" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand list-repo" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand list-repo" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand list-repo" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand list-repo" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand list-repo" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand list-sources" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand list-sources" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand list-sources" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand list-sources" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand list-sources" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand list-sources" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand list-sources" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand list-sources" -l json
complete -c pisi -n "__fish_pisi_using_subcommand list-sources" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand list-sources" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand list-sources" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand list-sources" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand list-sources" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand list-sources" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand list-sources" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand list-upgrades" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand list-upgrades" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand list-upgrades" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand list-upgrades" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand list-upgrades" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand list-upgrades" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand list-upgrades" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand list-upgrades" -l json
complete -c pisi -n "__fish_pisi_using_subcommand list-upgrades" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand list-upgrades" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand list-upgrades" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand list-upgrades" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand list-upgrades" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand list-upgrades" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand list-upgrades" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand rebuild-db" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand rebuild-db" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand rebuild-db" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand rebuild-db" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand rebuild-db" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand rebuild-db" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand rebuild-db" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand rebuild-db" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand rebuild-db" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand rebuild-db" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand rebuild-db" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand rebuild-db" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand rebuild-db" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand rebuild-db" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand remove" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand remove" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand remove" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand remove" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand remove" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand remove" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand remove" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand remove" -l ignore-dependency
complete -c pisi -n "__fish_pisi_using_subcommand remove" -l ignore-safety
complete -c pisi -n "__fish_pisi_using_subcommand remove" -l ignore-comar
complete -c pisi -n "__fish_pisi_using_subcommand remove" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand remove" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand remove" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand remove" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand remove" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand remove" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand remove" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand remove-orphaned" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand remove-orphaned" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand remove-orphaned" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand remove-orphaned" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand remove-orphaned" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand remove-orphaned" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand remove-orphaned" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand remove-orphaned" -l json
complete -c pisi -n "__fish_pisi_using_subcommand remove-orphaned" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand remove-orphaned" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand remove-orphaned" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand remove-orphaned" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand remove-orphaned" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand remove-orphaned" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand remove-orphaned" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand remove-repo" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand remove-repo" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand remove-repo" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand remove-repo" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand remove-repo" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand remove-repo" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand remove-repo" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand remove-repo" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand remove-repo" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand remove-repo" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand remove-repo" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand remove-repo" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand remove-repo" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand remove-repo" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand rollback" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand rollback" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand rollback" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand rollback" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand rollback" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand rollback" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand rollback" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand rollback" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand rollback" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand rollback" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand rollback" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand rollback" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand rollback" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand rollback" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand search" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand search" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand search" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand search" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand search" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand search" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand search" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand search" -l json
complete -c pisi -n "__fish_pisi_using_subcommand search" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand search" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand search" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand search" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand search" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand search" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand search" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand search-file" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand search-file" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand search-file" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand search-file" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand search-file" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand search-file" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand search-file" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand search-file" -l json
complete -c pisi -n "__fish_pisi_using_subcommand search-file" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand search-file" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand search-file" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand search-file" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand search-file" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand search-file" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand search-file" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand update-repo" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand update-repo" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand update-repo" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand update-repo" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand update-repo" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand update-repo" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand update-repo" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand update-repo" -l json
complete -c pisi -n "__fish_pisi_using_subcommand update-repo" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand update-repo" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand update-repo" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand update-repo" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand update-repo" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand update-repo" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand update-repo" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand upgrade" -l component -r
complete -c pisi -n "__fish_pisi_using_subcommand upgrade" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand upgrade" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand upgrade" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand upgrade" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand upgrade" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand upgrade" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand upgrade" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand upgrade" -l check-only
complete -c pisi -n "__fish_pisi_using_subcommand upgrade" -l integrity-only
complete -c pisi -n "__fish_pisi_using_subcommand upgrade" -l no-integrity
complete -c pisi -n "__fish_pisi_using_subcommand upgrade" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand upgrade" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand upgrade" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand upgrade" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand upgrade" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand upgrade" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand upgrade" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand version" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand version" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand version" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand version" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand version" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand version" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand version" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand version" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand version" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand version" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand version" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand version" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand version" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand version" -s h -l help -d 'Print help'
complete -c pisi -n "__fish_pisi_using_subcommand temp" -s D -l destdir -r
complete -c pisi -n "__fish_pisi_using_subcommand temp" -s L -l bandwidth-limit -r
complete -c pisi -n "__fish_pisi_using_subcommand temp" -s u -l username -r
complete -c pisi -n "__fish_pisi_using_subcommand temp" -s p -l password -r
complete -c pisi -n "__fish_pisi_using_subcommand temp" -s j -l jobs -r
complete -c pisi -n "__fish_pisi_using_subcommand temp" -l log-path -r -F
complete -c pisi -n "__fish_pisi_using_subcommand temp" -l opt-level -r
complete -c pisi -n "__fish_pisi_using_subcommand temp" -s y -l yes-all
complete -c pisi -n "__fish_pisi_using_subcommand temp" -s v -l verbose
complete -c pisi -n "__fish_pisi_using_subcommand temp" -s d -l debug
complete -c pisi -n "__fish_pisi_using_subcommand temp" -s N -l no-color
complete -c pisi -n "__fish_pisi_using_subcommand temp" -l download-only
complete -c pisi -n "__fish_pisi_using_subcommand temp" -l ignore-check
complete -c pisi -n "__fish_pisi_using_subcommand temp" -s h -l help -d 'Print help'
