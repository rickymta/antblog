package stat

import (
	"runtime"
	"syscall"
)

// Proc là ảnh chụp tài nguyên của chính tiến trình.
type Proc struct {
	CPUUserS   float64 `json:"cpu_user_s"`
	CPUSysS    float64 `json:"cpu_sys_s"`
	Goroutines int     `json:"goroutines"`
	HeapMB     float64 `json:"heap_alloc_mb"`
	SysMB      float64 `json:"go_sys_mb"`
	MaxRSSMB   float64 `json:"max_rss_mb"`
}

// ReadProc đọc getrusage và runtime.MemStats. Trên macOS ru_maxrss tính bằng byte.
func ReadProc() Proc {
	var ru syscall.Rusage
	_ = syscall.Getrusage(syscall.RUSAGE_SELF, &ru)
	var ms runtime.MemStats
	runtime.ReadMemStats(&ms)
	rss := float64(ru.Maxrss)
	if runtime.GOOS == "darwin" {
		rss /= 1 << 20
	} else {
		rss /= 1 << 10
	}
	return Proc{
		CPUUserS:   float64(ru.Utime.Sec) + float64(ru.Utime.Usec)/1e6,
		CPUSysS:    float64(ru.Stime.Sec) + float64(ru.Stime.Usec)/1e6,
		Goroutines: runtime.NumGoroutine(),
		HeapMB:     float64(ms.HeapAlloc) / (1 << 20),
		SysMB:      float64(ms.Sys) / (1 << 20),
		MaxRSSMB:   rss,
	}
}
