#ifndef TOI_NOI_ROUND_FARE_H_
#define TOI_NOI_ROUND_FARE_H_

#include <stdint.h>

#if defined(_WIN32)
#define TOI_NOI_EXPORT __declspec(dllexport)
#else
#define TOI_NOI_EXPORT __attribute__((visibility("default")))
#endif

/* Làm tròn cước (đồng) tới bội số gần nhất của `step`, nửa bước làm tròn lên.
 * Trả -1 khi đầu vào sai: vnd < 0, step <= 0, hoặc kết quả vượt INT64_MAX. */
TOI_NOI_EXPORT int64_t toi_noi_round_fare(int64_t vnd, int32_t step);

#endif /* TOI_NOI_ROUND_FARE_H_ */
