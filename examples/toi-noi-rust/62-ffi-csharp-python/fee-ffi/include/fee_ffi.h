/* Tới Nơi: C ABI của thư viện tính phí. Sinh bởi cbindgen, không sửa tay. */

#ifndef FEE_FFI_H
#define FEE_FFI_H

#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

// Phiên bản hợp đồng ABI. Đổi bố cục struct hay chữ ký hàm thì tăng số này.
#define FEE_ABI_VERSION 1

// Mã kết quả của mọi hàm xuất ra. `i32` cố định để C# ánh xạ sang `int`.
enum FeeStatus
#if defined(__cplusplus) || __STDC_VERSION__ >= 202311L
  : int32_t
#endif // defined(__cplusplus) || __STDC_VERSION__ >= 202311L
 {
  // Thành công.
  FeeStatus_Ok = 0,
  // Một con trỏ bắt buộc là null.
  FeeStatus_NullPointer = 1,
  // Giá trị không hợp lệ (cước âm, quãng đường 0, mã chuyến rỗng).
  FeeStatus_InvalidArgument = 2,
  // Chuỗi đầu vào không phải UTF-8.
  FeeStatus_InvalidUtf8 = 3,
  // Con trỏ không phải handle đang sống do thư viện này cấp (lạ, hoặc đã giải phóng).
  FeeStatus_BadHandle = 4,
  // Có panic bên trong và đã bị chặn lại ở ranh giới.
  FeeStatus_Panic = 99,
};
#ifndef __cplusplus
#if __STDC_VERSION__ >= 202311L
typedef enum FeeStatus FeeStatus;
#else
typedef int32_t FeeStatus;
#endif // __STDC_VERSION__ >= 202311L
#endif // __cplusplus

// Báo giá một chuyến, nằm trên heap của Rust. Phía gọi chỉ giữ con trỏ mờ (opaque), không biết bố cục.
typedef struct FeeQuote FeeQuote;

// Kết quả chia cước, bố cục C: ba `i64` liền nhau, 24 byte, căn lề 8.
typedef struct FeeSplit {
  // Cước khách trả, đồng.
  int64_t fare_dong;
  // Phí nền tảng, đồng.
  int64_t platform_fee_dong;
  // Phần tài xế nhận, đồng.
  int64_t driver_net_dong;
} FeeSplit;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

// Số `FeeQuote` và chuỗi do thư viện cấp mà phía gọi chưa trả lại. Dùng để kiểm rò rỉ.
uint64_t fee_live_allocations(void);

// Phiên bản ABI, để phía gọi từ chối nạp bản thư viện lệch.
uint32_t fee_abi_version(void);

// Chia cước `fare_dong` và ghi kết quả vào `*out`.
//
// # Safety
// `out` phải null (trả `NullPointer`) hoặc trỏ tới vùng ghi được, căn lề 8, đủ chỗ cho một `FeeSplit`.
FeeStatus fee_split(int64_t fare_dong,
                    struct FeeSplit *out);

// Tạo báo giá cho chuyến `trip_id` đi `km` km. Trả handle (con trỏ mờ), hoặc null khi lỗi.
// Mã lỗi ghi vào `*status` nếu `status` khác null. Phải giải phóng bằng [`fee_quote_free`].
//
// # Safety
// `trip_id` phải null (trả `NullPointer`) hoặc trỏ tới chuỗi kết thúc bằng NUL, đọc được suốt lời gọi.
// `status` phải null hoặc trỏ tới vùng ghi được, căn lề 4, đủ chỗ cho một `FeeStatus`.
struct FeeQuote *fee_quote_new(const char *trip_id,
                               uint32_t km,
                               FeeStatus *status);

// Chép phần chia cước của báo giá vào `*out`.
//
// # Safety
// `out` phải null (trả `NullPointer`) hoặc trỏ tới vùng ghi được, căn lề 8, đủ chỗ cho một `FeeSplit`.
// `quote` có thể là giá trị bất kỳ: handle không còn sống nhận `BadHandle`.
FeeStatus fee_quote_split(const struct FeeQuote *quote,
                          struct FeeSplit *out);

// Mô tả báo giá thành chuỗi UTF-8 do Rust cấp. Trả null khi handle không còn sống.
// Phía gọi phải trả chuỗi bằng [`fee_string_free`], không `free` bằng allocator của mình.
char *fee_quote_describe(const struct FeeQuote *quote);

// Giải phóng chuỗi do [`fee_quote_describe`] trả về. Gọi lần hai trên cùng con trỏ trả `BadHandle`.
FeeStatus fee_string_free(char *text);

// Giải phóng báo giá. Gọi lần hai trên cùng handle trả `BadHandle`, không chạm vào bộ nhớ.
FeeStatus fee_quote_free(struct FeeQuote *quote);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus

#endif  /* FEE_FFI_H */
