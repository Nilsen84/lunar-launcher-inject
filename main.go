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
			home, err := os.UserHomeDir()
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

func Run() (err error) {
	lunarExe, err := GetLunarExecutable()
	if err != nil {
		return fmt.Errorf("%w\nfailed to locate the lunar launcher executable, try passing it by argument", err)
	}

	d, cmd, err := StartProcessAndConnectDebugger(lunarExe)
	if cmd != nil {
		defer func() {
			if err != nil {
				_ = cmd.Process.Kill()
			}
		}()
	}
	if err != nil {
		return
	}
	defer d.Close()

	ex, err := os.Executable()
	if err != nil {
		return
	}

	return d.Send("Runtime.callFunctionOn", map[string]any{
		"executionContextId":  1,
		"functionDeclaration": injectJs,
		"arguments": []any{
			map[string]any{
				"value": filepath.Dir(ex),
			},
		},
	})
}

func main() {
	log.SetFlags(0)

	if err := Run(); err != nil {
		log.Fatalln(err)
	}
}

//go:embed inject.js
var injectJs string
