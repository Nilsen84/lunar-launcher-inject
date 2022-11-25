# lunar_launcher_inject
Small wrapper for lunar clients electron launcher.

Uses the [Chrome DevTools Protocol](https://chromedevtools.github.io/devtools-protocol/)
to override the node child_process.spawn function and inject jvm arguments. See [inject.js](src/inject.js).

Re-enables agent attaching + loads all jar files in the same directory as the executable as premain agents.

## Usage
* [Download the executable for your OS](https://github.com/Nilsen84/lunar_launcher_inject/releases)
* Place it in a directory together with any premain agents you want to use
* Launch lunar using this program instead of the official launcher. The launcher looks normal but will silently inject the agents on launch.
