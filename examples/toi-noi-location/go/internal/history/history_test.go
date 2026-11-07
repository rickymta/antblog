package history

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/parquet-go/parquet-go"

	"toinoi/location/internal/geo"

	pb "toinoi/location/gen/location/v1"
)

func u32(v uint32) *uint32 { return &v }

func event(driver, city string, t time.Time, northM float64, heading uint32, flags uint32) *pb.DriverLocationUpdated {
	lat, lng := geo.Offset(geo.FromDeg(10.78), geo.FromDeg(106.70), northM, 0)
	return &pb.DriverLocationUpdated{DriverId: driver, CityCode: city, OccurredAtMs: t.UnixMilli(), ReceivedAtMs: t.UnixMilli() + 300,
		Flags: flags, TripId: "TN-0001",
		Point: &pb.GpsPoint{LatE7: lat, LngE7: lng, Seq: uint32(t.Unix() % 1000), SpeedDmps: u32(0), HeadingDdeg: u32(heading), AccuracyDm: u32(80)}}
}

func TestGiamMau(t *testing.T) {
	s := NewSampler(DefaultDownsample())
	t0 := time.Date(2026, 11, 6, 3, 0, 0, 0, time.UTC)
	type step struct {
		sec     int
		north   float64
		heading uint32
		flags   uint32
		want    string
	}
	for i, st := range []step{
		{0, 0, 0, 0, "first"},
		{4, 5, 0, 0, ""},               // di chuyển 5 m
		{8, 10, 0, 0, ""},              // vẫn dưới 60 m
		{12, 70, 0, 0, "move"},         // 70 m từ điểm giữ
		{16, 72, 1800, 0, "turn"},      // quay đầu 180°, đã 4 s
		{20, 73, 1800, 0, ""},          // đứng yên
		{24, 73, 1800, 1, "flagged"},   // có cờ thì luôn giữ
		{60, 73, 1800, 0, ""},          // mới 36 s
		{84, 73, 1800, 0, "heartbeat"}, // 60 s kể từ điểm giữ cuối
		{86, 73, 1900, 0, ""},          // đổi hướng 10° nhưng chưa tới ngưỡng
	} {
		_, got := s.Keep(event("DR-001", "HCM", t0.Add(time.Duration(st.sec)*time.Second), st.north, st.heading, st.flags))
		if got != st.want {
			t.Errorf("bước %d (t=%ds): được %q, muốn %q", i, st.sec, got, st.want)
		}
	}
	// Tài xế khác không dùng chung trạng thái.
	if _, r := s.Keep(event("DR-002", "HCM", t0, 0, 0, 0)); r != "first" {
		t.Fatalf("tài xế mới phải là first, được %q", r)
	}
}

func TestPhanVungNgayTheoGioVietNam(t *testing.T) {
	root := t.TempDir()
	w := NewWriter(root, 1000, Anonymizer{Level: LevelRaw}, nil)
	// 18:00 UTC ngày 06 là 01:00 ngày 07 ở Việt Nam.
	_ = w.Add(event("DR-001", "HCM", time.Date(2026, 11, 6, 18, 0, 0, 0, time.UTC), 0, 0, 0))
	_ = w.Add(event("DR-002", "HN", time.Date(2026, 11, 6, 3, 0, 0, 0, time.UTC), 0, 0, 0))
	if err := w.Flush(); err != nil {
		t.Fatal(err)
	}
	for _, p := range []string{"date=2026-11-07/city=HCM", "date=2026-11-06/city=HN"} {
		m, _ := filepath.Glob(filepath.Join(root, p, "part-*.parquet"))
		if len(m) != 1 {
			t.Fatalf("thiếu file ở %s", p)
		}
	}
	if tmp, _ := filepath.Glob(filepath.Join(root, "*", "*", ".*")); len(tmp) != 0 {
		t.Fatalf("còn file tạm: %v", tmp)
	}
}

func TestDocLaiDungSchemaVaNull(t *testing.T) {
	root := t.TempDir()
	w := NewWriter(root, 1000, Anonymizer{Level: LevelRaw}, nil)
	ev := event("DR-001", "HCM", time.Date(2026, 11, 6, 3, 0, 0, 0, time.UTC), 0, 900, 4)
	noOpt := event("DR-001", "HCM", time.Date(2026, 11, 6, 3, 0, 4, 0, time.UTC), 5, 0, 0)
	noOpt.Point.SpeedDmps, noOpt.Point.AccuracyDm, noOpt.TripId = nil, nil, ""
	_ = w.Add(ev)
	_ = w.Add(noOpt)
	if err := w.Flush(); err != nil {
		t.Fatal(err)
	}
	files, _ := filepath.Glob(filepath.Join(root, "date=*", "city=*", "*.parquet"))
	rows, err := parquet.ReadFile[Row](files[0])
	if err != nil || len(rows) != 2 {
		t.Fatalf("đọc lại: %v, %d dòng", err, len(rows))
	}
	r := rows[0]
	if *r.DriverRef != "DR-001" || r.IDLevel != 0 || r.SpeedDmps == nil || *r.SpeedDmps != 0 || *r.HeadingDdeg != 900 || r.Flags != 4 ||
		r.EventTime != ev.OccurredAtMs || r.LatE7 != ev.Point.LatE7 || *r.TripID != "TN-0001" {
		t.Fatalf("dòng 0 sai: %+v", r)
	}
	if rows[1].SpeedDmps != nil || rows[1].AccuracyDm != nil || rows[1].TripID != nil {
		t.Fatalf("giá trị vắng phải là NULL, 0 không được lẫn với NULL: %+v", rows[1])
	}
	f, _ := os.Open(files[0])
	defer f.Close()
	st, _ := f.Stat()
	pf, _ := parquet.OpenFile(f, st.Size())
	if v, _ := pf.Lookup("toinoi.schema_version"); v != "1" {
		t.Fatalf("metadata schema_version = %q", v)
	}
}

func TestMucDinhDanh(t *testing.T) {
	key := []byte("khoa-lab")
	ev := event("DR-001", "HCM", time.Date(2026, 11, 6, 3, 0, 0, 0, time.UTC), 0, 0, 0)

	raw := Anonymizer{Level: LevelRaw, Key: key}.Row(ev, "all")
	if *raw.DriverRef != "DR-001" || raw.LatE7 != ev.Point.LatE7 {
		t.Fatal("mức raw giữ nguyên")
	}
	ps := Anonymizer{Level: LevelPseudonym, Key: key}
	a := ps.Row(ev, "all")
	if strings.Contains(*a.DriverRef, "DR-") || len(*a.DriverRef) != 16 || a.LatE7 != ev.Point.LatE7 {
		t.Fatalf("pseudonym %q", *a.DriverRef)
	}
	same := ps.Row(event("DR-001", "HCM", time.Date(2026, 11, 20, 3, 0, 0, 0, time.UTC), 0, 0, 0), "all")
	other := ps.Row(event("DR-002", "HCM", time.Date(2026, 11, 6, 3, 0, 0, 0, time.UTC), 0, 0, 0), "all")
	nextMonth := ps.Row(event("DR-001", "HCM", time.Date(2026, 12, 6, 3, 0, 0, 0, time.UTC), 0, 0, 0), "all")
	if *same.DriverRef != *a.DriverRef || *other.DriverRef == *a.DriverRef || *nextMonth.DriverRef == *a.DriverRef {
		t.Fatal("ổn định trong tháng, khác tài xế khác giá trị, khác tháng khác giá trị")
	}
	ev = event("DR-001", "HCM", time.Date(2026, 11, 6, 3, 0, 0, 0, time.UTC), 437, 0, 0) // toạ độ không tròn sẵn
	co := Anonymizer{Level: LevelCoarse, Key: key}.Row(ev, "all")
	if co.DriverRef != nil || co.TripID != nil {
		t.Fatal("mức coarse không có driver_ref và trip_id")
	}
	if d := geo.HaversineM(co.LatE7, co.LngE7, ev.Point.LatE7, ev.Point.LngE7); d > 100 || d == 0 {
		t.Fatalf("làm tròn thô lệch %.1f m, phải trong vài chục mét và khác 0", d)
	}
}

func TestRetentionXoaTheoNgayTrongTen(t *testing.T) {
	root := t.TempDir()
	for _, d := range []string{"2026-07-01", "2026-10-07", "2026-10-08", "2026-11-06"} {
		dir := filepath.Join(root, "date="+d, "city=HCM")
		_ = os.MkdirAll(dir, 0o755)
		_ = os.WriteFile(filepath.Join(dir, "part-1.parquet"), []byte("x"), 0o644)
	}
	_ = os.MkdirAll(filepath.Join(root, "khong-phai-ngay"), 0o755)
	now := time.Date(2026, 11, 6, 12, 0, 0, 0, vnZone)

	dry, err := Retention(root, 30, now, true)
	if err != nil || len(dry) != 2 || dry[0].Date != "2026-07-01" || dry[1].Date != "2026-10-07" {
		t.Fatalf("dry-run: %v %+v", err, dry)
	}
	if _, err := os.Stat(filepath.Join(root, "date=2026-07-01")); err != nil {
		t.Fatal("dry-run không được xoá")
	}
	if _, err := Retention(root, 30, now, false); err != nil {
		t.Fatal(err)
	}
	// 30 ngày gồm hôm nay: giữ từ 2026-10-08 trở đi.
	for d, want := range map[string]bool{"2026-07-01": false, "2026-10-07": false, "2026-10-08": true, "2026-11-06": true} {
		_, err := os.Stat(filepath.Join(root, "date="+d))
		if (err == nil) != want {
			t.Errorf("date=%s tồn tại=%v, muốn %v", d, err == nil, want)
		}
	}
	if _, err := os.Stat(filepath.Join(root, "khong-phai-ngay")); err != nil {
		t.Fatal("không được đụng thư mục lạ")
	}
}

func TestSaiSoGiamMau(t *testing.T) {
	// Đường thẳng đi bắc: bỏ điểm giữa không sai số. Điểm lệch 30 m sang đông phải đo ra ~30 m.
	mk := func(north, east float64, kept bool) Sample {
		lat, lng := geo.Offset(geo.FromDeg(10.78), geo.FromDeg(106.70), north, east)
		return Sample{LatE7: lat, LngE7: lng, Kept: kept}
	}
	errs := DropError([]Sample{mk(0, 0, true), mk(50, 0, false), mk(100, 30, false), mk(200, 0, true)})
	if len(errs) != 2 || errs[0] > 0.5 || errs[1] < 29 || errs[1] > 31 {
		t.Fatalf("sai số = %v", errs)
	}
}
