-- Rate limit cửa sổ cố định, nguyên tử: INCR và EXPIRE chạy trong một lệnh EVAL.
-- KEYS[1] = rl:<client>:<số cửa sổ>   ARGV[1] = thời gian sống (giây)   ARGV[2] = giới hạn
-- Trả về {được phép (1/0), số lần hiện tại}.
local current = redis.call('INCR', KEYS[1])
if current == 1 then
  redis.call('EXPIRE', KEYS[1], tonumber(ARGV[1]))
end
if current > tonumber(ARGV[2]) then
  return {0, current}
end
return {1, current}
