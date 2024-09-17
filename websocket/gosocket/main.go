package main

import (
	"bufio"
	"crypto/sha1"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io"
	"net"
	"strings"
)

func main() {
	fmt.Println("Hello world")

	server, err := net.Listen("tcp", ":8000")
	if err != nil {
		fmt.Println("Tcp didn't start correctly:", err)
		return
	}
	defer server.Close()

	for {
		conn, err := server.Accept()
		if err != nil {
			fmt.Println("Server didn't accept the connection correctly:", err)
			continue
		}

		go handleConnection(conn)
	}
}

func handleConnection(tcpConnection net.Conn) {
	defer tcpConnection.Close()

	reader := bufio.NewReader(tcpConnection)

	// Read the request line
	requestLine, err := reader.ReadString('\n')
	if err != nil {
		fmt.Println("Error reading request line:", err)
		return
	}

	requestLine = strings.TrimSpace(requestLine)

	// Read and parse headers
	headers := make(map[string]string)
	for {
		line, err := reader.ReadString('\n')
		if err != nil {
			fmt.Println("Error reading headers:", err)
			return
		}
		line = strings.TrimSpace(line)
		if line == "" {
			break // End of headers
		}
		parts := strings.SplitN(line, ":", 2)
		if len(parts) == 2 {
			headers[strings.TrimSpace(parts[0])] = strings.TrimSpace(parts[1])
		}
	}

	// Pass the request line and headers for matching
	patternMatcher(requestLine, headers, tcpConnection)
}

func patternMatcher(requestLine string, headers map[string]string, conn net.Conn) {
	fmt.Println("Request:", requestLine)

	switch requestLine {
	case "GET / HTTP/1.1":
		httpResponse(conn, "200 OK", "text/html", "<h1>This is the index</h1>")
	case "GET /random HTTP/1.1":
		httpResponse(conn, "200 OK", "text/html", "<h1>This is the random page!</h1>")
	case "GET /socketme HTTP/1.1":
		if headers["Upgrade"] == "websocket" && headers["Connection"] == "Upgrade" {
			websocketStarting(conn, headers["Sec-WebSocket-Key"])
		} else {
			httpResponse(conn, "400 Bad Request", "text/html", "<h1>400 Bad Request</h1>")
		}
	default:
		httpResponse(conn, "404 Not Found", "text/html", "<h1>404 Not Found</h1>")
	}
}

func httpResponse(conn net.Conn, status string, contentType string, body string) {
	response := fmt.Sprintf("HTTP/1.1 %s\r\n", status)
	response += fmt.Sprintf("Content-Type: %s\r\n", contentType)
	response += fmt.Sprintf("Content-Length: %d\r\n", len(body))
	response += "Connection: close\r\n" // Close the connection after response
	response += "\r\n"                  // End of headers
	response += body                    // Response body

	// Send the response back to the client
	fmt.Fprint(conn, response)
}

func websocketStarting(conn net.Conn, key string) {
	acceptKey := generateWebSocketAcceptKey(key)

	response := fmt.Sprintf("HTTP/1.1 101 Switching Protocols\r\n")
	response += "Upgrade: websocket\r\n"
	response += "Connection: Upgrade\r\n"
	response += "Sec-WebSocket-Accept: " + acceptKey + "\r\n"
	response += "\r\n" // End of headers

	// Send the response back to the client
	fmt.Fprint(conn, response)

	// The connection is now upgraded to WebSocket
	// Prepare JSON data to send
	data := []map[string]interface{}{
		{"name": "Alice", "score": 100},
		{"name": "Bob", "score": 200},
	}

	dataJSON, err := json.Marshal(data)
	if err != nil {
		fmt.Println("Error marshaling JSON:", err)
		return
	}

	// Create a WebSocket frame
	frame := createWebSocketFrame(dataJSON)

	// Send the frame to the client
	_, err = conn.Write(frame)
	if err != nil {
		fmt.Println("Error sending WebSocket frame:", err)
		return
	}
}

func generateWebSocketAcceptKey(key string) string {
	const magic = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11"
	h := sha1.New()
	io.WriteString(h, key+magic)
	return base64.StdEncoding.EncodeToString(h.Sum(nil))
}

func createWebSocketFrame(data []byte) []byte {
	frame := make([]byte, 0, len(data)+2)
	frame = append(frame, 0x81)            // FIN, Text frame
	frame = append(frame, byte(len(data))) // Payload length

	frame = append(frame, data...) // Payload data

	return frame
}
