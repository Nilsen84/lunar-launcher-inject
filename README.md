# lunar_launcher_inject
Small wrapper for lunar clients electron launcher.

Uses the [Chrome DevTools Protocol](https://chromedevtools.github.io/devtools-protocol/)
to override the node subprocess.spawn function and inject jvm arguments. See [inject.js](src/inject.js).

Re-enables runtime agent attaching + loads all jar files in the same directory as the executable as java agents.
