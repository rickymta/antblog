// Package history: luồng lịch sử GPS tách khỏi luồng nóng, ghi Parquet theo ngày và thành phố.
//
// Schema v1 là hợp đồng với phía Python (LOC-51): đổi tên cột, kiểu hay ý nghĩa là phá hợp đồng,
// chỉ được thêm cột mới ở cuối (và tăng SchemaVersion). Mô tả đầy đủ ở proto/location/v1/gps-history-parquet.md.
package history

import (
	"crypto/hmac"
	"crypto/sha256"
	"encoding/hex"
	"time"

	pb "toinoi/location/gen/location/v1"
)

// SchemaVersion ghi vào metadata từng file.
const SchemaVersion = 1

// IDLevel là mức định danh của dữ liệu trong một thư mục đầu ra.
type IDLevel int32

const (
	// LevelRaw: driver_ref là driver_id thật, toạ độ nguyên. Chỉ cho điều tra, tranh chấp; truy cập hạn chế, giữ ngắn.
	LevelRaw IDLevel = 0
	// LevelPseudonym: driver_ref là HMAC theo tháng của driver_id. Vẫn là dữ liệu cá nhân
	// (quỹ đạo đủ để nhận ra người), nhưng không đọc ra mã tài xế và không nối được qua tháng.
	LevelPseudonym IDLevel = 1
	// LevelCoarse: không có driver_ref, toạ độ làm tròn cỡ trăm mét. Dành cho nhu cầu theo khu vực và huấn luyện.
	LevelCoarse IDLevel = 2
)

// Row là một dòng Parquet. Con trỏ nil là NULL; 0 là giá trị thật (xe đang đứng yên có tốc độ 0).
type Row struct {
	EventTime    int64   `parquet:"event_time,timestamp(millisecond)"`
	ReceivedTime int64   `parquet:"received_time,timestamp(millisecond)"`
	DriverRef    *string `parquet:"driver_ref,dict"`
	IDLevel      int32   `parquet:"id_level"`
	TripID       *string `parquet:"trip_id,dict"`
	LatE7        int32   `parquet:"lat_e7"`
	LngE7        int32   `parquet:"lng_e7"`
	AccuracyDm   *int32  `parquet:"accuracy_dm"`
	SpeedDmps    *int32  `parquet:"speed_dmps"`
	HeadingDdeg  *int32  `parquet:"heading_ddeg"`
	Seq          int64   `parquet:"seq"`
	Flags        int32   `parquet:"flags"`
	SampleReason string  `parquet:"sample_reason,dict"`
}

// Anonymizer đổi một sự kiện thành Row theo mức định danh. Key là khoá gốc cho HMAC.
type Anonymizer struct {
	Level IDLevel
	Key   []byte
}

// coarseDrop là số chữ số E7 bị bỏ ở LevelCoarse: 10^4 đơn vị E7 = 0,001° ≈ 111 m theo vĩ độ.
const coarseDrop = 4

func roundE7(v int32, drop int) int32 {
	p := int32(1)
	for i := 0; i < drop; i++ {
		p *= 10
	}
	if v >= 0 {
		return (v + p/2) / p * p
	}
	return -((-v + p/2) / p * p)
}

func i32(v *uint32) *int32 {
	if v == nil {
		return nil
	}
	x := int32(*v)
	return &x
}

// Pseudonym là HMAC(HMAC(key, "YYYY-MM"), driver_id) rút gọn 8 byte. Đổi khoá theo tháng: cùng tài xế
// ở hai tháng khác nhau cho hai giá trị khác nhau.
func (a Anonymizer) Pseudonym(driverID string, t time.Time) string {
	m := hmac.New(sha256.New, a.Key)
	m.Write([]byte(t.UTC().Format("2006-01")))
	mk := m.Sum(nil)
	d := hmac.New(sha256.New, mk)
	d.Write([]byte(driverID))
	return hex.EncodeToString(d.Sum(nil)[:8])
}

// Row dựng dòng Parquet từ sự kiện; reason là lý do giữ điểm sau khi giảm mẫu.
func (a Anonymizer) Row(ev *pb.DriverLocationUpdated, reason string) Row {
	p := ev.Point
	r := Row{
		EventTime: ev.OccurredAtMs, ReceivedTime: ev.ReceivedAtMs, IDLevel: int32(a.Level),
		LatE7: p.LatE7, LngE7: p.LngE7, AccuracyDm: i32(p.AccuracyDm), SpeedDmps: i32(p.SpeedDmps),
		HeadingDdeg: i32(p.HeadingDdeg), Seq: int64(p.Seq), Flags: int32(ev.Flags), SampleReason: reason,
	}
	if ev.TripId != "" {
		t := ev.TripId
		r.TripID = &t
	}
	switch a.Level {
	case LevelRaw:
		d := ev.DriverId
		r.DriverRef = &d
	case LevelPseudonym:
		d := a.Pseudonym(ev.DriverId, time.UnixMilli(ev.OccurredAtMs))
		r.DriverRef = &d
	case LevelCoarse:
		r.LatE7, r.LngE7 = roundE7(r.LatE7, coarseDrop), roundE7(r.LngE7, coarseDrop)
		r.TripID = nil // chuyến cộng với quỹ đạo thô vẫn nối lại được với một người; bỏ ở mức này
	}
	return r
}
