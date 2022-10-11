# lunar_launcher_inject
Small wrapper for lunar clients electron launcher.

Uses the [Chrome DevTools Protocol](https://chromedevtools.github.io/devtools-protocol/)
to override the node child_process.spawn function and inject jvm arguments. See [inject.js](src/inject.js).

Re-enables agent attaching + loads all jar files in the same directory as the executable as premain agents.
