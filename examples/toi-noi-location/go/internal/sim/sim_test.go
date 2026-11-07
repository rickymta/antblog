package sim

import (
	"testing"

	"toinoi/location/internal/geo"
)

func TestXacDinhTheoSeed(t *testing.T) {
	a, b := NewFleet(50, 7), NewFleet(50, 7)
	for i := range a {
		pa, pb := a[i].Step(4, 1000), b[i].Step(4, 1000)
		if a[i].ID != b[i].ID || a[i].City != b[i].City || pa.LatE7 != pb.LatE7 || pa.LngE7 != pb.LngE7 {
			t.Fatalf("tài xế %d lệch giữa hai lần chạy", i)
		}
	}
	if c := NewFleet(50, 8); c[0].Step(4, 1000).LatE7 == a[0].Step(4, 2000).LatE7 && c[1].City == a[1].City && c[2].City == a[2].City {
		t.Log("seed khác nhưng trùng một vài giá trị, chấp nhận được")
	}
}

func TestBuocDiHopLy(t *testing.T) {
	d := NewFleet(1, 1)[0]
	p0 := d.Step(4, 0)
	for i := 1; i < 200; i++ {
		p := d.Step(4, int64(i)*4000)
		// 14 m/s × 4 s = 56 m cộng nhiễu tối đa vài chục mét.
		if dist := geo.HaversineM(p0.LatE7, p0.LngE7, p.LatE7, p.LngE7); dist > 200 {
			t.Fatalf("bước %d nhảy %.0f m", i, dist)
		}
		p0 = p
	}
}
