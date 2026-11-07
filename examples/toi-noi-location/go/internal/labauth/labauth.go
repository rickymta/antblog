// Package labauth: token HMAC đơn giản cho LAB.
//
// CHỈ DÙNG CHO LAB. Production dùng JWT do Identity service ký (khoá bất đối xứng,
// xoay khoá, thu hồi) và kiểm tra ở gateway. Ở đây chỉ cần chứng minh được luồng:
// token gắn tài xế với kết nối, và khách chỉ nghe được chuyến của mình.
package labauth

import (
	"crypto/hmac"
	"crypto/sha256"
	"encoding/base64"
	"encoding/json"
	"errors"
	"strings"
	"time"
)

// Vai trò trong token.
const (
	RoleDriver   = "driver"
	RoleCustomer = "customer"
	RoleAdmin    = "admin" // chỉ để lab gọi /lab/assign thay cho sự kiện của Dispatch
)

// Claims là phần nội dung được ký.
type Claims struct {
	Sub  string `json:"sub"`            // DR-001, CU-001
	Role string `json:"role"`           // driver | customer | admin
	City string `json:"city,omitempty"` // HCM, HN...
	Trip string `json:"trip,omitempty"` // khách xin nghe chuyến nào
	Exp  int64  `json:"exp"`            // epoch giây
}

var (
	ErrFormat  = errors.New("token sai định dạng")
	ErrSig     = errors.New("chữ ký không khớp")
	ErrExpired = errors.New("token hết hạn")
)

var enc = base64.RawURLEncoding

func mac(secret []byte, payload string) []byte {
	h := hmac.New(sha256.New, secret)
	h.Write([]byte(payload))
	return h.Sum(nil)
}

// Sign tạo token dạng <payload>.<chữ ký>.
func Sign(secret []byte, c Claims) string {
	b, _ := json.Marshal(c)
	p := enc.EncodeToString(b)
	return p + "." + enc.EncodeToString(mac(secret, p))
}

// Verify kiểm chữ ký bằng so sánh thời gian hằng số rồi kiểm hạn.
func Verify(secret []byte, token string, now time.Time) (Claims, error) {
	var c Claims
	p, s, ok := strings.Cut(token, ".")
	if !ok {
		return c, ErrFormat
	}
	sig, err := enc.DecodeString(s)
	if err != nil {
		return c, ErrFormat
	}
	if !hmac.Equal(sig, mac(secret, p)) {
		return c, ErrSig
	}
	raw, err := enc.DecodeString(p)
	if err != nil || json.Unmarshal(raw, &c) != nil {
		return c, ErrFormat
	}
	if now.Unix() >= c.Exp {
		return c, ErrExpired
	}
	return c, nil
}

// Bearer lấy token từ header Authorization.
func Bearer(h string) string {
	return strings.TrimSpace(strings.TrimPrefix(h, "Bearer "))
}
