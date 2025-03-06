To compile on windows so it doesn't show the terminal, use this

For release build: ```cargo rustc --release -- -Clink-args="/SUBSYSTEM:WINDOWS /ENTRY:mainCRTStartup"```

For debug build: ```cargo rustc -- -Clink-args="/SUBSYSTEM:WINDOWS /ENTRY:mainCRTStartup"```

or for gcc toolchain

Release: ```cargo rustc --release -- -Clink-args="-Wl,--subsystem,windows"```

Debug: ```cargo rustc -- -Clink-args="-Wl,--subsystem,windows"```
