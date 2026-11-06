package history

import (
	"math"
	"sort"

	"toinoi/location/internal/geo"
)

// Sample là một điểm của quỹ đạo gốc kèm quyết định giữ hay bỏ.
type Sample struct {
	LatE7, LngE7 int32
	TimeMs       int64
	Kept         bool
}

// DropError đo chất lượng giảm mẫu: với mỗi điểm bị bỏ, khoảng cách (mét) tới đoạn thẳng nối hai điểm
// được giữ liền kề. Đây là sai số nếu người dùng vẽ lại quỹ đạo từ dữ liệu đã giảm mẫu. Cần samples theo thứ tự thời gian.
func DropError(samples []Sample) []float64 {
	var out []float64
	last := -1
	for i, s := range samples {
		if !s.Kept {
			continue
		}
		if last >= 0 {
			for j := last + 1; j < i; j++ {
				out = append(out, distToSegmentM(samples[j], samples[last], s))
			}
		}
		last = i
	}
	return out
}

// distToSegmentM dùng phép chiếu phẳng quanh điểm a, đủ chính xác cho đoạn vài trăm mét.
func distToSegmentM(p, a, b Sample) float64 {
	cosLat := math.Cos(geo.ToDeg(a.LatE7) * math.Pi / 180)
	const mPerDeg = 111_320.0
	xy := func(s Sample) (float64, float64) {
		return (geo.ToDeg(s.LngE7) - geo.ToDeg(a.LngE7)) * mPerDeg * cosLat, (geo.ToDeg(s.LatE7) - geo.ToDeg(a.LatE7)) * mPerDeg
	}
	px, py := xy(p)
	bx, by := xy(b)
	l2 := bx*bx + by*by
	t := 0.0
	if l2 > 0 {
		t = math.Max(0, math.Min(1, (px*bx+py*by)/l2))
	}
	return math.Hypot(px-t*bx, py-t*by)
}

// Quantile trả phân vị q (0..1) của xs; xs bị sắp xếp tại chỗ.
func Quantile(xs []float64, q float64) float64 {
	if len(xs) == 0 {
		return 0
	}
	sort.Float64s(xs)
	return xs[int(math.Min(float64(len(xs)-1), math.Ceil(q*float64(len(xs)))-1))]
}
