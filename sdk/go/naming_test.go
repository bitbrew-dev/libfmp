package fmp

import (
	"go/ast"
	"go/parser"
	"go/token"
	"path/filepath"
	"strings"
	"testing"
)

// goSpelling mirrors the gen_go initialism table (emit.rs WORDS, ADR 0032):
// the Go spelling of each word that is not plainly capitalised, keyed by
// the lowercase word.
var goSpelling = map[string]string{
	"8k": "8K", "13f": "13F", "api": "API", "cik": "CIK", "ciks": "CIKs",
	"cot": "COT", "cusip": "CUSIP", "dcf": "DCF", "eps": "EPS", "esg": "ESG",
	"etf": "ETF", "etfs": "ETFs", "fmp": "FMP", "http": "HTTP", "id": "ID",
	"ids": "IDs", "ipo": "IPO", "ipos": "IPO", "isin": "ISIN", "json": "JSON",
	"sec": "SEC", "sp500": "SP500", "tipranks": "TipRanks", "ttm": "TTM",
	"uid": "UID", "url": "URL", "urls": "URLs", "us": "US",
}

// camelWords splits an identifier the way gen_go's camel_words does: a word
// starts at an uppercase letter after a lowercase letter or digit, at the
// last capital of an uppercase run followed by a lowercase letter, and at a
// digit run that follows a letter and ends in a lowercase letter.
func camelWords(name string) []string {
	var words []string
	start := 0
	for i := 1; i < len(name); i++ {
		prev, cur := name[i-1], name[i]
		upper := isUpper(cur) && (isLower(prev) || isDigit(prev) ||
			(isUpper(prev) && i+1 < len(name) && isLower(name[i+1])))
		digit := false
		if isDigit(cur) && (isUpper(prev) || isLower(prev)) {
			end := i
			for end < len(name) && isDigit(name[end]) {
				end++
			}
			digit = end < len(name) && isLower(name[end])
		}
		if upper || digit {
			words = append(words, name[start:i])
			start = i
		}
	}
	return append(words, name[start:])
}

func isUpper(c byte) bool { return 'A' <= c && c <= 'Z' }
func isLower(c byte) bool { return 'a' <= c && c <= 'z' }
func isDigit(c byte) bool { return '0' <= c && c <= '9' }

func TestCamelWordsMatchesTheGenerator(t *testing.T) {
	t.Parallel()
	cases := map[string]string{
		"Form13fFilingDate":  "Form 13f Filing Date",
		"Latest8KSECFilings": "Latest8 KSEC Filings",
		"Sp500Constituent":   "Sp500 Constituent",
		"CIKListing":         "CIK Listing",
		"PriceAvg50":         "Price Avg50",
		"MutualFundsAndEtfs": "Mutual Funds And Etfs",
	}
	for name, want := range cases {
		if got := strings.Join(camelWords(name), " "); got != want {
			t.Errorf("camelWords(%q) = %q, want %q", name, got, want)
		}
	}
}

// TestExportedIdentifiersUseGoInitialisms scans every exported identifier
// of the package's non-test files (types, functions, methods, constants,
// variables, and struct fields, generated or hand-written) and fails on an
// initialism spelled in mixed case, such as Fmp, Id, Url, Json, or Ttm.
func TestExportedIdentifiersUseGoInitialisms(t *testing.T) {
	t.Parallel()
	files, err := filepath.Glob("*.go")
	if err != nil {
		t.Fatal(err)
	}
	fset := token.NewFileSet()
	scanned := 0
	for _, path := range files {
		if strings.HasSuffix(path, "_test.go") {
			continue
		}
		file, err := parser.ParseFile(fset, path, nil, parser.SkipObjectResolution)
		if err != nil {
			t.Fatal(err)
		}
		ast.Inspect(file, func(node ast.Node) bool {
			var idents []*ast.Ident
			switch n := node.(type) {
			case *ast.FuncDecl:
				idents = append(idents, n.Name)
			case *ast.TypeSpec:
				idents = append(idents, n.Name)
			case *ast.ValueSpec:
				idents = append(idents, n.Names...)
			case *ast.Field:
				idents = append(idents, n.Names...)
			}
			for _, ident := range idents {
				if !ident.IsExported() {
					continue
				}
				scanned++
				for _, word := range camelWords(ident.Name) {
					if want, ok := goSpelling[strings.ToLower(word)]; ok && word != want {
						t.Errorf("%s: %s spells %q, want %q",
							fset.Position(ident.Pos()), ident.Name, word, want)
					}
				}
			}
			return true
		})
	}
	if scanned < 2000 {
		t.Fatalf("scanned %d exported identifiers, want the whole package", scanned)
	}
}
