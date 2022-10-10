function _(...agents) {
    const cp = require('child_process'), originalSpawn = cp.spawn;

    // noinspection JSValidateTypes
    cp.spawn = function (cmd, args, opts){
        args = args.filter(e => e !== '-XX:+DisableAttachMechanism')

        delete opts.env['_JAVA_OPTIONS']
        delete opts.env['JAVA_TOOL_OPTIONS']
        delete opts.env['JDK_JAVA_OPTIONS']

        return originalSpawn(
            cmd,
            [...agents.map(a => '-javaagent:' + a), ...args],
            opts
        )
    }
}