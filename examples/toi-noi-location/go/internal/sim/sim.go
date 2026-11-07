// Package sim sinh tài xế giả di chuyển ngẫu nhiên quanh tâm 6 thành phố. Có seed nên chạy lại ra đúng
// quỹ đạo. Không có dữ liệu GPS người thật: tâm thành phố chỉ là điểm xuất phát công khai.
package sim

import (
	"fmt"
	"math"
	"math/rand/v2"

	"toinoi/location/internal/geo"

	pb "toinoi/location/gen/location/v1"
)

// City là thành phố và tỷ trọng đội xe (tổng 100).
type City struct {
	Code     string
	Lat, Lng float64
	Weight   int
}

// Cities gồm 6 thành phố của Tới Nơi, toạ độ tâm làm tròn.
var Cities = []City{
	{"HCM", 10.78, 106.70, 40}, {"HN", 21.03, 105.85, 30}, {"DN", 16.05, 108.21, 10},
	{"CT", 10.03, 105.78, 8}, {"HP", 20.86, 106.68, 7}, {"NT", 12.24, 109.19, 5},
}

// Driver là một tài xế mô phỏng; mỗi goroutine sở hữu riêng một Driver nên không cần khoá.
type Driver struct {
	ID, City string
	lat, lng int32
	heading  float64 // radian
	speed    float64 // m/s
	seq      uint32
	rng      *rand.Rand
}

// DriverID đặt tên theo quy ước DR-nnnnn.
func DriverID(i int) string { return fmt.Sprintf("DR-%05d", i+1) }

// NewFleet tạo n tài xế, hoàn toàn xác định theo seed.
func NewFleet(n int, seed uint64) []*Driver {
	root := rand.New(rand.NewPCG(seed, 0x9e3779b97f4a7c15))
	out := make([]*Driver, n)
	for i := range out {
		r := rand.New(rand.NewPCG(seed, uint64(i)+1))
		pick, c := root.IntN(100), Cities[0]
		for _, cc := range Cities {
			if pick < cc.Weight {
				c = cc
				break
			}
			pick -= cc.Weight
		}
		// Rải trong khoảng ±6 km quanh tâm.
		lat, lng := geo.Offset(geo.FromDeg(c.Lat), geo.FromDeg(c.Lng), (r.Float64()-.5)*12000, (r.Float64()-.5)*12000)
		out[i] = &Driver{ID: DriverID(i), City: c.Code, lat: lat, lng: lng,
			heading: r.Float64() * 2 * math.Pi, speed: 4 + r.Float64()*8, rng: r}
	}
	return out
}

// Step đi tiếp dt giây rồi trả một điểm GPS có nhiễu, gắn giờ nowMs. Tốc độ 0–14 m/s (xe máy trong phố).
func (d *Driver) Step(dt float64, nowMs int64) *pb.GpsPoint {
	d.heading += d.rng.NormFloat64() * 0.25
	d.speed = math.Max(0, math.Min(14, d.speed+d.rng.NormFloat64()*1.5))
	if d.rng.IntN(25) == 0 {
		d.speed = 0 // đèn đỏ
	}
	d.lat, d.lng = geo.Offset(d.lat, d.lng, math.Cos(d.heading)*d.speed*dt, math.Sin(d.heading)*d.speed*dt)
	acc := uint32(30 + d.rng.IntN(220)) // 3–25 m
	sigma := float64(acc) / 10 / 3
	lat, lng := geo.Offset(d.lat, d.lng, d.rng.NormFloat64()*sigma, d.rng.NormFloat64()*sigma)
	d.seq++
	sp := uint32(d.speed * 10)
	hd := uint32(math.Mod(d.heading*180/math.Pi+3600, 360) * 10)
	return &pb.GpsPoint{LatE7: lat, LngE7: lng, DeviceTimeMs: nowMs, Seq: d.seq,
		AccuracyDm: &acc, SpeedDmps: &sp, HeadingDdeg: &hd}
}

// Teleport dịch vị trí thật đi km kilômét, mô phỏng spoofing hoặc lỗi GPS để thử cờ.
func (d *Driver) Teleport(km float64) { d.lat, d.lng = geo.Offset(d.lat, d.lng, km*1000, 0) }
