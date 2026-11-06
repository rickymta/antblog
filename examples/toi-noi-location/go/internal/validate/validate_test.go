package validate

import (
	"testing"

	"toinoi/location/internal/geo"

	pb "toinoi/location/gen/location/v1"
)

const recv = int64(1_800_000_000_000) // giờ server cố định cho test

func u32(v uint32) *uint32 { return &v }

func pt(latOffM, lngOffM float64, tMs int64, seq uint32) *pb.GpsPoint {
	lat, lng := geo.Offset(geo.FromDeg(10.78), geo.FromDeg(106.70), latOffM, lngOffM)
	return &pb.GpsPoint{LatE7: lat, LngE7: lng, DeviceTimeMs: tMs, Seq: seq, AccuracyDm: u32(80)}
}

func TestBangDiemXau(t *testing.T) {
	cfg := Default()
	base := recv - 1000 // điểm nền, đã có trong State
	cases := []struct {
		name   string
		mod    func(p *pb.GpsPoint)
		skewMs int64
		reason Reason
		flags  []pb.PointFlag
		clean  []pb.PointFlag // cờ không được có
	}{
		{name: "điểm tốt", mod: func(p *pb.GpsPoint) {}, clean: []pb.PointFlag{pb.PointFlag_POINT_FLAG_TELEPORT, pb.PointFlag_POINT_FLAG_SPEED_IMPLAUSIBLE, pb.PointFlag_POINT_FLAG_SPEED_MISMATCH}},
		{name: "toạ độ ngoài miền", mod: func(p *pb.GpsPoint) { p.LatE7 = 95 * geo.E7 }, reason: BadCoord},
		{name: "null island", mod: func(p *pb.GpsPoint) { p.LatE7, p.LngE7 = 0, 0 }, reason: BadCoord},
		{name: "giờ ở tương lai", mod: func(p *pb.GpsPoint) { p.DeviceTimeMs = recv + 60_000 }, reason: Future},
		{name: "quá cũ", mod: func(p *pb.GpsPoint) { p.DeviceTimeMs = recv - 10*60_000 }, reason: TooOld},
		{name: "đến sau điểm mới hơn", mod: func(p *pb.GpsPoint) { p.DeviceTimeMs = base - 500 }, reason: OutOfOrder},
		{name: "trùng giờ trùng seq", mod: func(p *pb.GpsPoint) { p.DeviceTimeMs = base; p.Seq = 1 }, reason: Duplicate},
		{name: "seq về 0 nhưng giờ mới (app khởi động lại)", mod: func(p *pb.GpsPoint) { p.Seq = 0 }},
		{name: "đồng hồ nhanh 90 s: hiệu chỉnh và gắn cờ", skewMs: -90_000,
			mod:   func(p *pb.GpsPoint) { p.DeviceTimeMs = recv + 90_000 - 500 },
			flags: []pb.PointFlag{pb.PointFlag_POINT_FLAG_CLOCK_SKEW}},
		{name: "accuracy 250 m", mod: func(p *pb.GpsPoint) { p.AccuracyDm = u32(2500) }, flags: []pb.PointFlag{pb.PointFlag_POINT_FLAG_POOR_ACCURACY}},
		{name: "app khai báo giả lập", mod: func(p *pb.GpsPoint) { p.IsMock = true }, flags: []pb.PointFlag{pb.PointFlag_POINT_FLAG_MOCK_REPORTED}},
		{name: "tốc độ khai báo 80 m/s", mod: func(p *pb.GpsPoint) { p.SpeedDmps = u32(800) }, flags: []pb.PointFlag{pb.PointFlag_POINT_FLAG_SPEED_IMPLAUSIBLE}},
		{name: "nhảy 3 km trong 1 s", mod: func(p *pb.GpsPoint) {
			q := pt(3000, 0, recv-500, 2)
			p.LatE7, p.LngE7 = q.LatE7, q.LngE7
		}, flags: []pb.PointFlag{pb.PointFlag_POINT_FLAG_TELEPORT, pb.PointFlag_POINT_FLAG_SPEED_IMPLAUSIBLE}},
		{name: "nhảy 200 m trong 1 s: trong ngưỡng teleport nhưng quá nhanh", mod: func(p *pb.GpsPoint) {
			q := pt(200, 0, recv-500, 2)
			p.LatE7, p.LngE7 = q.LatE7, q.LngE7
		}, flags: []pb.PointFlag{pb.PointFlag_POINT_FLAG_SPEED_IMPLAUSIBLE}, clean: []pb.PointFlag{pb.PointFlag_POINT_FLAG_TELEPORT}},
		{name: "nhảy 30 m trong 1 s (jitter, trong sai số)", mod: func(p *pb.GpsPoint) {
			q := pt(30, 0, recv-500, 2)
			p.LatE7, p.LngE7 = q.LatE7, q.LngE7
		}, clean: []pb.PointFlag{pb.PointFlag_POINT_FLAG_SPEED_IMPLAUSIBLE, pb.PointFlag_POINT_FLAG_TELEPORT}},
		{name: "khai báo đứng yên nhưng đã đi 100 m trong 5 s", mod: func(p *pb.GpsPoint) {
			q := pt(100, 0, base+5000, 2)
			p.LatE7, p.LngE7, p.DeviceTimeMs, p.SpeedDmps = q.LatE7, q.LngE7, q.DeviceTimeMs, u32(0)
		}, flags: []pb.PointFlag{pb.PointFlag_POINT_FLAG_SPEED_MISMATCH}},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			st := &State{}
			if v := cfg.Check(st, pt(0, 0, base, 1), 0, recv); v.Reason != Accepted {
				t.Fatalf("điểm nền bị loại: %v", v.Reason)
			}
			p := pt(5, 0, recv-500, 2) // cách 5 m, sau nửa giây
			tc.mod(p)
			v := cfg.Check(st, p, tc.skewMs, recv)
			if v.Reason != tc.reason {
				t.Fatalf("lý do = %q, muốn %q", v.Reason, tc.reason)
			}
			if v.Reason != Accepted {
				return
			}
			for _, f := range tc.flags {
				if !Has(v.Flags, f) {
					t.Errorf("thiếu cờ %v (flags=%b implied=%.1f)", f, v.Flags, v.Implied)
				}
			}
			for _, f := range tc.clean {
				if Has(v.Flags, f) {
					t.Errorf("không được có cờ %v (implied=%.1f)", f, v.Implied)
				}
			}
		})
	}
}

func TestSkewChiHieuChinhKhiVuotNguong(t *testing.T) {
	cfg := Default()
	if s := cfg.Skew(recv-800, recv); s != 0 {
		t.Fatalf("độ trễ mạng 0,8 s không phải lệch đồng hồ, được %d", s)
	}
	if s := cfg.Skew(recv+90_000, recv); s != -90_000 {
		t.Fatalf("điện thoại nhanh 90 s, được %d", s)
	}
}

func TestChuoiNhayRoiVeNoiGocGanCoCaHaiLan(t *testing.T) {
	// Mẫu giả mạo hay gặp: vị trí thật, nhảy sang chỗ khác, rồi quay lại. Cả hai lần nhảy phải có cờ.
	cfg := Default()
	st := &State{}
	cfg.Check(st, pt(0, 0, recv-3000, 1), 0, recv)
	a := cfg.Check(st, pt(5000, 0, recv-2000, 2), 0, recv)
	b := cfg.Check(st, pt(0, 0, recv-1000, 3), 0, recv)
	if !Has(a.Flags, pb.PointFlag_POINT_FLAG_TELEPORT) || !Has(b.Flags, pb.PointFlag_POINT_FLAG_TELEPORT) {
		t.Fatalf("cờ a=%b b=%b", a.Flags, b.Flags)
	}
}
