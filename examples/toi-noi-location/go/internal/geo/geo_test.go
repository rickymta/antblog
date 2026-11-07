package geo

import (
	"math"
	"testing"
)

func TestHaversineVaOffset(t *testing.T) {
	lat, lng := FromDeg(10.78), FromDeg(106.70)
	lat2, lng2 := Offset(lat, lng, 1000, 0)
	if d := HaversineM(lat, lng, lat2, lng2); math.Abs(d-1000) > 1 {
		t.Fatalf("1 km về phía bắc, đo được %.2f m", d)
	}
	lat3, lng3 := Offset(lat, lng, 0, 1000)
	if d := HaversineM(lat, lng, lat3, lng3); math.Abs(d-1000) > 1 {
		t.Fatalf("1 km về phía đông, đo được %.2f m", d)
	}
}

func TestE7KhuLamTron(t *testing.T) {
	// Một đơn vị E7 ở vĩ độ Việt Nam khoảng 1,1 cm.
	d := HaversineM(FromDeg(10.78), FromDeg(106.70), FromDeg(10.78)+1, FromDeg(106.70))
	if d < 0.009 || d > 0.013 {
		t.Fatalf("1 đơn vị E7 = %.4f m", d)
	}
}
