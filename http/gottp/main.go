package main

import (
	"bufio"
	"fmt"
	"net"
	"strings"
)

func main() {
	fmt.Println("Hello world")

	server, err := net.Listen("tcp", ":8000")
	if err != nil {
		fmt.Println("Tcp didn't start correctly")
	}

	for {

		conn, err := server.Accept()
		if err != nil {
			fmt.Println("Server didn't accept correctly the connection")
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

	// Read and ignore headers for now
	for {
		line, err := reader.ReadString('\n')
		if err != nil {
			fmt.Println("Error reading headers:", err)
			return
		}
		if line == "\r\n" || line == "\n" {
			break // End of headers
		}
	}

	// Pass the request line for matching
	patternMatcher(requestLine, tcpConnection)
}

func patternMatcher(requestLine string, conn net.Conn) {

	fmt.Println("Request:", requestLine)

	switch requestLine {
	case "GET / HTTP/1.1":
		httpResponse(conn, "200 OK", "text/html", "<h1>This is the index</h1>")
	case "GET /random HTTP/1.1":
		httpResponse(conn, "200 OK", "text/html", "<h1>This is the random page !</h1>")
	default:
		httpResponse(conn, "404 Not found", "text/html", "<h1>404 Not Found</h1>")
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
