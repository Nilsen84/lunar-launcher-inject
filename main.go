package main

import (
	_ "embed"
	"fmt"
	"log"
	"os"
	"path/filepath"
	"runtime"
)

func GetLunarExecutable() (string, error) {
	var exe string

	if len(os.Args) > 1 {
		exe = os.Args[1]
	} else {
		switch runtime.GOOS {
		case "windows":
			var home, err = os.UserHomeDir()
			if err != nil {
				return "", err
			}
			exe = home + `\AppData\Local\Programs\lunarclient\Lunar Client.exe`
		case "macos":
			exe = "/Applications/Lunar Client.app/Contents/MacOS/Lunar Client"
		case "linux":
			exe = "/usr/bin/lunarclient"
		default:
			return "", fmt.Errorf("locating lunar is not supported on %s", runtime.GOOS)
		}
	}

	if _, err := os.Stat(exe); err == nil {
		return exe, nil
	}

	return "", fmt.Errorf("'%s' does not exist", exe)
}

func GetExecutableDirectory() (string, error) {
	ex, err := os.Executable()
	if err != nil {
		return "", err
	}

	return filepath.Dir(ex), nil
}

//go:embed inject.js
var injectJs string

func main() {
	log.SetFlags(0)

	lunarExe, err := GetLunarExecutable()
	if err != nil {
		log.Println(err)
		log.Fatalln("failed to locate the lunar launcher executable, try passing it by argument")
	}

	d, cmd, err := StartProcessAndConnectDebugger(lunarExe)
	if err != nil {
		if cmd != nil {
			_ = cmd.Process.Kill()
		}

		log.Fatalln(err)
	}
	defer d.Close()
	dir, err := GetExecutableDirectory()
	if err != nil {
		_ = cmd.Process.Kill()
		log.Fatalln(err)
	}

	err = d.Send("Runtime.callFunctionOn", map[string]any{
		"functionDeclaration": injectJs,
		"arguments": []any{
			map[string]any{
				"value": dir,
			},
		},
		"executionContextId": 1,
	})

	if err != nil {
		_ = cmd.Process.Kill()
		log.Fatalln(err)
	}
}
