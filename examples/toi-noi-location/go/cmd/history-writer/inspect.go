package main

import (
	"flag"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"sort"
	"strings"

	"github.com/parquet-go/parquet-go"

	"toinoi/location/internal/history"
)

// inspect đọc lại mọi file Parquet dưới --out: in cột, metadata, số dòng theo phân vùng.
func inspect(args []string) error {
	f := flag.NewFlagSet("inspect", flag.ExitOnError)
	_, out := anonFlags(f)
	_ = f.Parse(args)
	perPart := map[string][2]int64{} // số dòng, số byte
	var total int64
	var schemaShown bool
	err := filepath.WalkDir(*out, func(p string, d fs.DirEntry, err error) error {
		if err != nil || d.IsDir() || !strings.HasSuffix(p, ".parquet") || strings.HasPrefix(d.Name(), ".") {
			return err
		}
		fh, err := os.Open(p)
		if err != nil {
			return err
		}
		defer fh.Close()
		st, _ := fh.Stat()
		pf, err := parquet.OpenFile(fh, st.Size())
		if err != nil {
			return fmt.Errorf("%s: %w", p, err)
		}
		if !schemaShown {
			schemaShown = true
			fmt.Println("schema (file đầu tiên):")
			for _, c := range pf.Schema().Fields() {
				req := "NOT NULL"
				if c.Optional() {
					req = "NULLABLE"
				}
				fmt.Printf("  %-14s %-8s %s\n", c.Name(), req, c.Type())
			}
			sv, _ := pf.Lookup("toinoi.schema_version")
			lv, _ := pf.Lookup("toinoi.id_level")
			fmt.Printf("metadata: toinoi.schema_version=%s toinoi.id_level=%s\n", sv, lv)
			// Chỉ in cột định danh, không in toạ độ cùng định danh.
			if rows, err := parquet.ReadFile[history.Row](p); err == nil && len(rows) > 1 {
				ref := func(r history.Row) string {
					if r.DriverRef == nil {
						return "NULL"
					}
					return *r.DriverRef
				}
				fmt.Printf("driver_ref của hai dòng đầu: %s, %s (id_level=%d)\n", ref(rows[0]), ref(rows[1]), rows[0].IDLevel)
			}
		}
		rel, _ := filepath.Rel(*out, filepath.Dir(p))
		v := perPart[rel]
		perPart[rel] = [2]int64{v[0] + pf.NumRows(), v[1] + st.Size()}
		total += pf.NumRows()
		return nil
	})
	if err != nil {
		return err
	}
	keys := make([]string, 0, len(perPart))
	for k := range perPart {
		keys = append(keys, k)
	}
	sort.Strings(keys)
	for _, k := range keys {
		fmt.Printf("%-28s rows=%-8d bytes=%d\n", k, perPart[k][0], perPart[k][1])
	}
	fmt.Printf("tổng: %d dòng trong %d phân vùng\n", total, len(keys))
	return nil
}
