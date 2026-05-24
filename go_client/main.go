package main

import (
	"bufio"
	"fmt"
	"net"
	"os"
	"strings"
)

func main() {
	// Require the user to pass a database command
	if len(os.Args) < 2 {
		fmt.Println("Usage: go run main.go [COMMAND] [ARGS...]")
		os.Exit(1)
	}

	// Connect to the Rust server
	conn, err := net.Dial("tcp", "127.0.0.1:6379")
	if err != nil {
		fmt.Println("Error connecting to database:", err)
		os.Exit(1)
	}
	defer conn.Close()

	// Capture the terminal arguments and join them into a single string
	message := strings.Join(os.Args[1:], " ")
	
	// Send the command to Rust
	_, err = conn.Write([]byte(message))
	if err != nil {
		fmt.Println("Error sending data:", err)
		return
	}

	// Wait for and read the response from Rust
	response, _ := bufio.NewReader(conn).ReadString('\n')
	fmt.Print("Response: ", response)
}