// Package geo chứa vài phép tính tối thiểu trên toạ độ E7 (độ × 10^7, int32).
// Bản đầy đủ nằm ở geo-core của phần Rust; ở đây chỉ đủ cho kiểm tra và mô phỏng.
package geo

import "math"

const (
	// E7 là hệ số đổi độ sang đơn vị lưu trữ.
	E7           = 1e7
	earthRadiusM = 6371008.8
)

// FromDeg đổi độ sang E7, làm tròn gần nhất.
func FromDeg(deg float64) int32 { return int32(math.Round(deg * E7)) }

// ToDeg đổi E7 về độ.
func ToDeg(e7 int32) float64 { return float64(e7) / E7 }

// HaversineM trả khoảng cách chim bay (mét) giữa hai điểm E7.
func HaversineM(lat1, lng1, lat2, lng2 int32) float64 {
	p1, p2 := ToDeg(lat1)*math.Pi/180, ToDeg(lat2)*math.Pi/180
	dp := p2 - p1
	dl := (ToDeg(lng2) - ToDeg(lng1)) * math.Pi / 180
	a := math.Sin(dp/2)*math.Sin(dp/2) + math.Cos(p1)*math.Cos(p2)*math.Sin(dl/2)*math.Sin(dl/2)
	return 2 * earthRadiusM * math.Asin(math.Min(1, math.Sqrt(a)))
}

// Offset dịch một điểm E7 theo hướng bắc/đông (mét), đủ chính xác cho vài km.
func Offset(lat, lng int32, northM, eastM float64) (int32, int32) {
	dLat := northM / earthRadiusM * 180 / math.Pi
	dLng := eastM / (earthRadiusM * math.Cos(ToDeg(lat)*math.Pi/180)) * 180 / math.Pi
	return lat + FromDeg(dLat), lng + FromDeg(dLng)
}
