-- Rate limit cửa sổ trượt bằng sorted set. Một key duy nhất nên an toàn khi dùng cluster mode.
-- KEYS[1] = rl:sw:<client>
-- ARGV[1] = thời điểm hiện tại (mili giây)  ARGV[2] = độ dài cửa sổ (mili giây)
-- ARGV[3] = giới hạn                         ARGV[4] = member duy nhất cho lần gọi này
local now = tonumber(ARGV[1])
local window = tonumber(ARGV[2])
local limit = tonumber(ARGV[3])

redis.call('ZREMRANGEBYSCORE', KEYS[1], 0, now - window)
local count = redis.call('ZCARD', KEYS[1])
if count >= limit then
  return {0, count}
end
redis.call('ZADD', KEYS[1], now, ARGV[4])
redis.call('PEXPIRE', KEYS[1], window)
return {1, count + 1}
