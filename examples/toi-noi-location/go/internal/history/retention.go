package history

import (
	"io/fs"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"time"
)

// Removed là một phân vùng ngày đã (hoặc sẽ) bị xoá.
type Removed struct {
	Date  string
	Files int
	Bytes int64
}

// Retention xoá mọi thư mục date=YYYY-MM-DD cũ hơn days ngày so với now (tính theo ngày giờ Việt Nam).
// Quyết định dựa trên NGÀY TRONG TÊN THƯ MỤC, không dựa mtime: copy hay khôi phục file không làm dữ liệu «trẻ lại».
// dryRun chỉ liệt kê. Ngày giữ lại gồm cả ngày hôm nay và days-1 ngày trước đó.
func Retention(root string, days int, now time.Time, dryRun bool) ([]Removed, error) {
	cutoff := now.In(vnZone).AddDate(0, 0, -(days - 1)).Format("2006-01-02") // thư mục có date < cutoff bị xoá
	ents, err := os.ReadDir(root)
	if err != nil {
		return nil, err
	}
	var out []Removed
	for _, e := range ents {
		d, ok := strings.CutPrefix(e.Name(), "date=")
		if !e.IsDir() || !ok {
			continue
		}
		if _, err := time.Parse("2006-01-02", d); err != nil || d >= cutoff {
			continue
		}
		r := Removed{Date: d}
		_ = filepath.WalkDir(filepath.Join(root, e.Name()), func(_ string, de fs.DirEntry, _ error) error {
			if de != nil && !de.IsDir() {
				r.Files++
				if info, err := de.Info(); err == nil {
					r.Bytes += info.Size()
				}
			}
			return nil
		})
		if !dryRun {
			if err := os.RemoveAll(filepath.Join(root, e.Name())); err != nil {
				return out, err
			}
		}
		out = append(out, r)
	}
	sort.Slice(out, func(i, j int) bool { return out[i].Date < out[j].Date })
	return out, nil
}
