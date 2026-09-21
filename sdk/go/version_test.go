package fmp

import (
	"bufio"
	"bytes"
	"errors"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

// workspaceVersion returns the [workspace.package] version from the Cargo.toml
// at path, mirroring the first awk pass of scripts/set-version.sh: only the
// first `version = "..."` line inside that section counts.
func workspaceVersion(path string) (string, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return "", err
	}

	inSection := false
	scanner := bufio.NewScanner(bytes.NewReader(data))
	for scanner.Scan() {
		line := scanner.Text()
		if strings.HasPrefix(line, "[") {
			inSection = strings.TrimSpace(line) == "[workspace.package]"
			continue
		}
		if !inSection {
			continue
		}
		key, value, ok := strings.Cut(line, "=")
		if !ok || strings.TrimSpace(key) != "version" {
			continue
		}
		value = strings.TrimSpace(value)
		if len(value) < 2 || value[0] != '"' || value[len(value)-1] != '"' {
			return "", fmt.Errorf("workspace version is not a quoted string: %q", line)
		}
		return value[1 : len(value)-1], nil
	}
	if err := scanner.Err(); err != nil {
		return "", err
	}
	return "", errors.New("no version under [workspace.package]")
}

// TestVersionMatchesWorkspace pins Version to the Cargo workspace version so
// the two cannot drift: the release commit rewrites both through
// scripts/set-version.sh, and the sdk/go/vX.Y.Z tag is immutable once the
// module proxy has served it. The test skips only when Cargo.toml is absent,
// which happens when the module is tested from the module cache rather than
// from a repository checkout.
func TestVersionMatchesWorkspace(t *testing.T) {
	t.Parallel()

	path := filepath.Join("..", "..", "Cargo.toml")
	want, err := workspaceVersion(path)
	if errors.Is(err, fs.ErrNotExist) {
		t.Skipf("%s not found: not a repository checkout", path)
	}
	if err != nil {
		t.Fatalf("read workspace version from %s: %v", path, err)
	}
	if Version != want {
		t.Fatalf("Version = %q, workspace version in %s = %q: run scripts/set-version.sh", Version, path, want)
	}
	if !strings.HasPrefix(defaultUserAgent, "libfmp-go/"+want) {
		t.Fatalf("defaultUserAgent = %q does not carry the workspace version %q", defaultUserAgent, want)
	}
}

func TestWorkspaceVersionParsesOnlyThePackageSection(t *testing.T) {
	t.Parallel()

	dir := t.TempDir()
	path := filepath.Join(dir, "Cargo.toml")
	content := "[workspace]\nmembers = [\"a\"]\n\n[workspace.dependencies]\nversion = \"1.0.0\"\n\n[workspace.package]\nedition = \"2024\"\nversion = \"3.2.1\"\nversion = \"9.9.9\"\n"
	if err := os.WriteFile(path, []byte(content), 0o600); err != nil {
		t.Fatal(err)
	}
	got, err := workspaceVersion(path)
	if err != nil {
		t.Fatal(err)
	}
	if got != "3.2.1" {
		t.Fatalf("workspaceVersion = %q, want %q", got, "3.2.1")
	}

	missing := filepath.Join(dir, "missing.toml")
	if _, err := workspaceVersion(missing); !errors.Is(err, fs.ErrNotExist) {
		t.Fatalf("missing file: err = %v, want fs.ErrNotExist", err)
	}

	noVersion := filepath.Join(dir, "no-version.toml")
	if err := os.WriteFile(noVersion, []byte("[workspace.package]\nedition = \"2024\"\n"), 0o600); err != nil {
		t.Fatal(err)
	}
	if _, err := workspaceVersion(noVersion); err == nil {
		t.Fatal("no version line: expected an error")
	}
}
