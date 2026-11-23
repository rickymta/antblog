# tom-tat.awk: tóm tắt log của bao-gia theo ngày.
# Dùng: awk -f tom-tat.awk FILE...   hoặc   zcat -f FILE... | awk -f tom-tat.awk
# Mỗi ngày một dòng: số yêu cầu, số lỗi, số yêu cầu chậm và thời gian trung bình
# của các yêu cầu thành công (ms).

$4 ~ /^q-/ {
  ngay = $1
  tong[ngay]++
  if ($3 == "ERROR") loi[ngay]++
  else if ($3 == "WARN") cham[ngay]++
  else { ms[ngay] += $(NF - 1); ok[ngay]++ }
}

END {
  printf "%-10s %8s %5s %5s %7s\n", "ngày", "yêu cầu", "lỗi", "chậm", "tb ms"
  fflush()
  for (ngay in tong)
    printf "%-10s %8d %5d %5d %7.1f\n", ngay, tong[ngay], loi[ngay], cham[ngay], ms[ngay] / ok[ngay] | "sort"
}
