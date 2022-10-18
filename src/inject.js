function _(appFolder) {
    const fs = require('fs')
    const path = require('path')

    function jars(){
        return fs.readdirSync(appFolder, { withFileTypes: true })
            .filter(f => f.isFile() && f.name.endsWith('jar'))
            .map(f => path.join(appFolder, f.name))
    }


    const cp = require('child_process'), originalSpawn = cp.spawn;
    // noinspection JSValidateTypes
    cp.spawn = function (cmd, args, opts){
        args = args.filter(e => e !== '-XX:+DisableAttachMechanism')

        delete opts.env['_JAVA_OPTIONS']
        delete opts.env['JAVA_TOOL_OPTIONS']
        delete opts.env['JDK_JAVA_OPTIONS']

        return originalSpawn(
            cmd,
            [...jars().map(a => '-javaagent:' + a), ...args],
            opts
        )
    }
}