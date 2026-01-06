package main

import (
	"bufio"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

func main() {
	if len(os.Args) < 2 {
		printUsage()
		os.Exit(1)
	}

	loadEnvFile()

	if os.Getenv("DATABASE_URL") == "" {
		fmt.Println("Error: DATABASE_URL environment variable is not set")
		os.Exit(1)
	}

	command := os.Args[1]
	if command != "up" && command != "down" && command != "status" && command != "fresh" {
		fmt.Printf("Unknown command: %s\n", command)
		printUsage()
		os.Exit(1)
	}

	fmt.Printf("Running migration: %s\n", command)

	projectRoot := filepath.Join("..", "..")
	migrationDir := filepath.Join(projectRoot, "migration")

	cmd := exec.Command("cargo", "run", "--", command)
	cmd.Dir = migrationDir
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	cmd.Stdin = os.Stdin
	cmd.Env = os.Environ()

	if err := cmd.Run(); err != nil {
		fmt.Printf("Migration failed: %v\n", err)
		os.Exit(1)
	}

	fmt.Println("Migration completed successfully")
}

func loadEnvFile() {
	envPath := filepath.Join("..", "..", ".env")
	file, err := os.Open(envPath)
	if err != nil {
		return
	}
	defer file.Close()

	scanner := bufio.NewScanner(file)
	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}
		parts := strings.SplitN(line, "=", 2)
		if len(parts) == 2 {
			key := strings.TrimSpace(parts[0])
			value := strings.TrimSpace(parts[1])
			if os.Getenv(key) == "" {
				os.Setenv(key, value)
			}
		}
	}
}

func printUsage() {
	fmt.Println("Usage: migrate <command>")
	fmt.Println()
	fmt.Println("Commands:")
	fmt.Println("  up      Apply pending migrations")
	fmt.Println("  down    Rollback last migration")
	fmt.Println("  status  Show migration status")
	fmt.Println("  fresh   Drop all tables and re-run migrations")
}
