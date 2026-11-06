package labauth

import (
	"testing"
	"time"
)

func TestSignVerify(t *testing.T) {
	sec := []byte("khoa-lab")
	now := time.Unix(1_000_000, 0)
	tok := Sign(sec, Claims{Sub: "DR-001", Role: RoleDriver, City: "HCM", Exp: now.Add(time.Hour).Unix()})

	c, err := Verify(sec, tok, now)
	if err != nil || c.Sub != "DR-001" || c.City != "HCM" {
		t.Fatalf("token hợp lệ bị từ chối: %v %+v", err, c)
	}
	if _, err := Verify([]byte("khoa-khac"), tok, now); err != ErrSig {
		t.Fatalf("khoá khác phải ErrSig, được %v", err)
	}
	if _, err := Verify(sec, tok, now.Add(2*time.Hour)); err != ErrExpired {
		t.Fatalf("hết hạn phải ErrExpired, được %v", err)
	}
	if _, err := Verify(sec, "rac", now); err != ErrFormat {
		t.Fatalf("sai định dạng phải ErrFormat, được %v", err)
	}
	// Sửa payload nhưng giữ chữ ký cũ phải bị chặn.
	forged := Sign(sec, Claims{Sub: "DR-999", Role: RoleDriver, Exp: now.Add(time.Hour).Unix()})
	if _, err := Verify(sec, forged[:len(forged)-3]+"AAA", now); err == nil {
		t.Fatal("chữ ký bị sửa vẫn qua")
	}
}
