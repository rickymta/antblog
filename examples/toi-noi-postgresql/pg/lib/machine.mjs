// Thông tin máy để ghi kèm kết quả đo (CPU, RAM, hệ điều hành, ổ đĩa chứa thư mục tạm).
import os from 'node:os';
import { spawnSync } from 'node:child_process';
import { parse } from 'node:path';

export function machineInfo() {
  const cpus = os.cpus();
  const info = {
    cpu: cpus[0]?.model?.trim() ?? '?',
    nhan: cpus.length,
    ram_gb: Math.round(os.totalmem() / 1073741824),
    he_dieu_hanh: `${os.version?.() ?? os.type()} (${os.platform()} ${os.release()}, ${os.arch()})`,
    node: process.version,
    thu_muc_tam: os.tmpdir(),
    o_dia: 'chưa xác định',
  };
  if (process.platform === 'win32') {
    const drive = parse(os.tmpdir()).root.replace(/[\\:]/g, '');
    const ps = `$d = Get-Partition -DriveLetter ${drive} | Get-Disk; ` +
      `$p = Get-PhysicalDisk | Where-Object DeviceId -eq $d.Number; ` +
      `'{0} | {1} | {2} | {3} GB' -f $d.FriendlyName, $p.MediaType, $p.BusType, [math]::Round($d.Size / 1GB)`;
    const r = spawnSync('powershell.exe', ['-NoProfile', '-NonInteractive', '-Command', ps], { encoding: 'utf8', windowsHide: true, timeout: 30000 });
    if (r.status === 0 && r.stdout.trim()) info.o_dia = `${drive}: ${r.stdout.trim()}`;
  } else {
    const r = spawnSync('df', ['-h', os.tmpdir()], { encoding: 'utf8' });
    if (r.status === 0) info.o_dia = r.stdout.trim().split('\n').at(-1);
  }
  return info;
}

/**
 * Tải CPU toàn máy (%) trung bình trong ~3 giây — ghi kèm kết quả đo để biết máy có
 * đang bận việc khác không. Windows: bộ đếm hiệu năng; nơi khác: loadavg 1 phút / số nhân.
 */
export function cpuLoad() {
  if (process.platform === 'win32') {
    const ps = "(Get-Counter '\\Processor(_Total)\\% Processor Time' -SampleInterval 1 -MaxSamples 3 | ForEach-Object { $_.CounterSamples.CookedValue } | Measure-Object -Average).Average";
    const r = spawnSync('powershell.exe', ['-NoProfile', '-NonInteractive', '-Command', ps], { encoding: 'utf8', windowsHide: true, timeout: 30000 });
    const v = Number(r.stdout.trim());
    return Number.isFinite(v) ? Math.round(v) : null;
  }
  return Math.round((os.loadavg()[0] / os.cpus().length) * 100);
}

/**
 * Đỉnh tổng bộ nhớ RIÊNG (private bytes) của các tiến trình `pids`, lấy mẫu mỗi
 * `everyMs` trong `seconds` giây (chạy song song với tải). Windows: một tiến trình
 * PowerShell lặp Get-Process. Nơi khác: trả null (chưa hỗ trợ — ghi "chưa đo").
 * Trả Promise<{ dinh, mau }> (byte, số mẫu) hoặc null.
 */
export async function privateMemoryPeak(pids, seconds, everyMs = 300) {
  if (process.platform !== 'win32' || !pids.length) return null;
  const { spawn } = await import('node:child_process');
  const n = Math.max(1, Math.floor((seconds * 1000) / everyMs));
  const ps = `$m = 0; $k = 0; for ($i = 0; $i -lt ${n}; $i++) { ` +
    `$s = (Get-Process -Id ${pids.join(',')} -ErrorAction SilentlyContinue | Measure-Object -Property PrivateMemorySize64 -Sum).Sum; ` +
    `if ($s -gt $m) { $m = $s }; $k++; Start-Sleep -Milliseconds ${everyMs} }; "$m $k"`;
  return new Promise((resolve) => {
    const p = spawn('powershell.exe', ['-NoProfile', '-NonInteractive', '-Command', ps], { windowsHide: true });
    let out = '';
    p.stdout.on('data', (d) => { out += d; });
    p.on('exit', () => {
      const [m, k] = out.trim().split(/\s+/).map(Number);
      resolve(Number.isFinite(m) && m > 0 ? { dinh: m, mau: k } : null);
    });
  });
}

/**
 * Bộ nhớ RIÊNG (private bytes) của các tiến trình có PID cho trước, tính bằng byte.
 * Windows: PowerShell Get-Process (PrivateMemorySize64). Linux: /proc/<pid>/status (VmRSS
 * — gồm cả phần shared_buffers đã chạm, ghi rõ khi báo cáo). Trả null nếu không đo được.
 */
export function privateMemory(pids) {
  if (!pids.length) return null;
  if (process.platform === 'win32') {
    const ps = `(Get-Process -Id ${pids.join(',')} -ErrorAction SilentlyContinue | Measure-Object -Property PrivateMemorySize64 -Sum).Sum`;
    const r = spawnSync('powershell.exe', ['-NoProfile', '-NonInteractive', '-Command', ps], { encoding: 'utf8', windowsHide: true, timeout: 30000 });
    const v = Number(r.stdout.trim());
    return Number.isFinite(v) && v > 0 ? v : null;
  }
  return null;
}
