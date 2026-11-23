# Trả lời hộp thoại chọn file của Windows mà SSH Studio mở ra: gõ đường dẫn rồi Enter.
#   powershell -NoProfile -File chon-file.ps1 -Path C:\...\an-laptop        chọn file
#   powershell -NoProfile -File chon-file.ps1 -Cancel                          đóng mọi hộp thoại đang mở
# Hộp thoại được nhận ra theo tiến trình ssh-studio và tiêu đề khác "SSH Studio".
param([string]$Path, [switch]$Cancel, [int]$TimeoutSec = 10)

Add-Type -AssemblyName System.Windows.Forms
Add-Type @"
using System; using System.Text; using System.Runtime.InteropServices;
public class Win {
  public delegate bool E(IntPtr h, IntPtr p);
  [DllImport("user32.dll")] public static extern bool EnumWindows(E e, IntPtr p);
  [DllImport("user32.dll")] public static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern int GetClassName(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern IntPtr PostMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
}
"@

function Find-Dialogs {
  $ids = @(Get-Process ssh-studio -ErrorAction SilentlyContinue | ForEach-Object { $_.Id })
  $found = New-Object System.Collections.ArrayList
  [Win]::EnumWindows({ param($h, $p)
      if ([Win]::IsWindowVisible($h)) {
        $cls = New-Object Text.StringBuilder 64; [Win]::GetClassName($h, $cls, 64) | Out-Null
        $pid2 = 0; [Win]::GetWindowThreadProcessId($h, [ref]$pid2) | Out-Null
        if ($ids -contains [int]$pid2 -and $cls.ToString() -eq '#32770') { [void]$found.Add($h) }
      }
      $true }, [IntPtr]::Zero) | Out-Null
  return $found
}

if ($Cancel) {
  foreach ($h in (Find-Dialogs)) { [Win]::PostMessage($h, 0x0010, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null }
  exit 0
}

$deadline = (Get-Date).AddSeconds($TimeoutSec)
do {
  $d = @(Find-Dialogs)
  if ($d.Count -gt 0) { break }
  Start-Sleep -Milliseconds 300
} while ((Get-Date) -lt $deadline)
if ($d.Count -eq 0) { Write-Error 'Không thấy hộp thoại chọn file'; exit 1 }

[Win]::SetForegroundWindow($d[0]) | Out-Null
Start-Sleep -Milliseconds 600
# Ô tên file có focus sẵn khi hộp thoại mở. Dán đường dẫn qua clipboard thay vì gõ từng phím,
# để bộ gõ tiếng Việt (EVKey, Unikey…) đang bật không biến đổi ký tự.
[System.Windows.Forms.Clipboard]::SetText($Path)
[System.Windows.Forms.SendKeys]::SendWait('^a')
[System.Windows.Forms.SendKeys]::SendWait('^v')
Start-Sleep -Milliseconds 300
[System.Windows.Forms.SendKeys]::SendWait('{ENTER}')
