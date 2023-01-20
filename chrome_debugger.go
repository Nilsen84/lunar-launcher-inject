package main

import (
	"encoding/json"
	"errors"
	"fmt"
	"github.com/avast/retry-go/v4"
	"github.com/gorilla/websocket"
	"github.com/phayes/freeport"
	"io"
	"net/http"
	"os/exec"
	"time"
)

type ChromeDebugger struct {
	conn *websocket.Conn
}

func (d *ChromeDebugger) Close() error {
	return d.conn.Close()
}

func (d *ChromeDebugger) Send(method string, params map[string]interface{}) error {
	return d.conn.WriteJSON(map[string]interface{}{
		"id":     1,
		"method": method,
		"params": params,
	})
}

func StartProcessAndConnectDebugger(program string) (*ChromeDebugger, *exec.Cmd, error) {
	port, err := freeport.GetFreePort()
	if err != nil {
		return nil, nil, err
	}

	cmd := exec.Command(program, fmt.Sprintf("--remote-debugging-port=%d", port))
	if err = cmd.Start(); err != nil {
		return nil, nil, err
	}

	d, err := ConnectDebugger(port)
	if err != nil {
		return nil, cmd, err
	}

	return d, cmd, nil
}

func ConnectDebugger(port int) (*ChromeDebugger, error) {
	url, err := getWebsocketUrl(port)
	if err != nil {
		return nil, err
	}

	c, _, err := websocket.DefaultDialer.Dial(url, nil)
	if err != nil {
		return nil, err
	}

	return &ChromeDebugger{
		conn: c,
	}, nil
}

func getWebsocketUrl(port int) (string, error) {
	var url string

	err := retry.Do(
		func() error {
			r, err := http.Get(fmt.Sprintf("http://localhost:%d/json/list", port))
			if err != nil {
				return err
			}

			defer r.Body.Close()

			body, err := io.ReadAll(r.Body)
			if err != nil {
				return err
			}

			var targets []struct {
				WebsocketUrl string `json:"webSocketDebuggerUrl"`
			}

			if err = json.Unmarshal(body, &targets); err != nil {
				return err
			} else if len(targets) == 0 {
				return errors.New("no debugging targets found")
			}

			url = targets[0].WebsocketUrl
			return nil
		},
		retry.Attempts(3),
		retry.DelayType(retry.FixedDelay),
		retry.Delay(350*time.Millisecond),
		retry.LastErrorOnly(true),
	)

	if err != nil {
		return "", err
	}

	return url, nil
}
