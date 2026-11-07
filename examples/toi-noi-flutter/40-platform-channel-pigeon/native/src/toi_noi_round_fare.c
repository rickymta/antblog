#include "toi_noi_round_fare.h"

int64_t toi_noi_round_fare(int64_t vnd, int32_t step) {
  if (vnd < 0 || step <= 0) {
    return -1;
  }
  const int64_t s = (int64_t)step;
  const int64_t rest = vnd % s;
  const int64_t down = vnd - rest;
  /* rest * 2 không tràn: rest < s <= INT32_MAX. */
  if (rest * 2 < s) {
    return down;
  }
  if (down > INT64_MAX - s) {
    return -1;
  }
  return down + s;
}
