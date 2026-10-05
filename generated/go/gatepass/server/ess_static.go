// generated from gatepass v1
// model digest 7d021b6ebe1c4715096f165d6564389be0f46311f67d791ed748f627314d611c
// contract digest 2668f3034afb388a33d7add462e15a830b6010fbfe83101f1dd2526fa18d52ed
// do not edit: regenerate with `ess synthesize`

// Static fallback, confined to the selected root.
package server

import (
	"net/http"
	"os"
	"path/filepath"
	"strings"
)

func memoryStaticRoot(root string) (string, error) {
	if root == "" {
		return "", nil
	}
	resolved, err := filepath.EvalSymlinks(root)
	if err != nil {
		return "", err
	}
	resolved, err = filepath.Abs(resolved)
	if err != nil {
		return "", err
	}
	info, err := os.Stat(resolved)
	if err != nil {
		return "", err
	}
	if !info.IsDir() {
		return "", &os.PathError{Op: "static", Path: root, Err: os.ErrInvalid}
	}
	return resolved, nil
}

func memoryStatic(writer http.ResponseWriter, request *http.Request, root string) {
	if request.Method != "GET" && request.Method != "HEAD" {
		writer.WriteHeader(405)
		return
	}
	decoded := request.URL.Path
	if !strings.HasPrefix(decoded, "/") || strings.ContainsAny(decoded, "\\\x00") {
		http.NotFound(writer, request)
		return
	}
	for _, segment := range strings.Split(decoded, "/") {
		if segment == ".." {
			http.NotFound(writer, request)
			return
		}
	}
	path, err := filepath.EvalSymlinks(filepath.Join(root, strings.TrimPrefix(decoded, "/")))
	if err != nil {
		http.NotFound(writer, request)
		return
	}
	info, err := os.Stat(path)
	if err != nil {
		http.NotFound(writer, request)
		return
	}
	if info.IsDir() {
		path, err = filepath.EvalSymlinks(filepath.Join(path, "index.html"))
		if err != nil {
			http.NotFound(writer, request)
			return
		}
	}
	relative, err := filepath.Rel(root, path)
	if err != nil || relative == ".." || strings.HasPrefix(relative, ".."+string(os.PathSeparator)) {
		http.NotFound(writer, request)
		return
	}
	file, err := os.Open(path)
	if err != nil {
		http.NotFound(writer, request)
		return
	}
	defer file.Close()
	info, err = file.Stat()
	if err != nil || !info.Mode().IsRegular() {
		http.NotFound(writer, request)
		return
	}
	http.ServeContent(writer, request, info.Name(), info.ModTime(), file)
}
