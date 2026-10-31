# Background processes started through `wsl.exe` die when it exits; launch long-lived workers with `systemd-run --user`

From the Windows session, `wsl.exe -d NixOS -- bash -lc '… setsid nohup … &'` printed PIDs, but the children
were gone (no log files even created) as soon as `wsl.exe` returned. `systemd-run --user --unit=fw16-<lane>
--collect bash -l <script>` survives; check with `systemctl --user list-units 'fw16-*'`. See
`_build/agents/launch.sh`. Also: quoting through `wsl.exe` mangles `$var` and globs — put commands in a script
file and run `wsl.exe -d NixOS -- bash -l /path/script.sh` (with `MSYS_NO_PATHCONV=1` from Git Bash).
