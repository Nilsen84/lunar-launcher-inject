(function (agentDirectory, endpoint) {
    const fs = require('fs')
    const path = require('path')

    function getAgentArgs() {
        const nativeExtension = { win32: '.dll', linux: '.so', darwin: '.dylib' }[process.platform]
        return fs.readdirSync(agentDirectory, {withFileTypes: true})
            .filter(f => f.isFile() && (f.name.endsWith('.jar') || path.extname(f.name) === nativeExtension))
            .map(f => (f.name.endsWith('.jar') ? '-javaagent:' : '-agentpath:') + path.join(agentDirectory, f.name))
    }

    const cp = require('child_process'), originalSpawn = cp.spawn

    cp.spawn = function (cmd, args, opts) {
        if (!['java', 'javaw'].includes(path.basename(cmd, '.exe'))) {
            return Reflect.apply(originalSpawn, this, arguments)
        }

        if (!Array.isArray(args)) {
            if (args != null) opts = args
            args = []
        }
        
        args = args.filter(e => e !== '-XX:+DisableAttachMechanism')
        delete opts?.env?._JAVA_OPTIONS
        delete opts?.env?.JAVA_TOOL_OPTIONS
        delete opts?.env?.JDK_JAVA_OPTIONS

        return originalSpawn.call(
            this,
            cmd,
            [...getAgentArgs(), ...args],
            opts
        )
    }
})
