# Shows the state of the virtual deck on this Windows machine: process, OpenDeck windows, config,
# which window is on top of the deck, a pixel summary and a screenshot.
# Usage: powershell -NoProfile -File deck-status.ps1 [-OutDir <folder for the screenshot>]
param([string]$OutDir = $env:TEMP)

$ErrorActionPreference = 'Continue'

Add-Type @"
using System; using System.Text; using System.Runtime.InteropServices; using System.Collections.Generic;
public static class DeckStatusWin {
  public delegate bool EnumProc(IntPtr h, IntPtr l);
  public struct RECT { public int L, T, R, B; }
  public struct POINT { public int X, Y; }
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc p, IntPtr l);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll")] public static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern long GetWindowLongPtr(IntPtr h, int i);
  [DllImport("user32.dll")] public static extern IntPtr WindowFromPoint(POINT p);
  [DllImport("user32.dll")] public static extern IntPtr GetAncestor(IntPtr h, uint f);
  [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr v);

  public static string Describe(IntPtr h) {
    var s = new StringBuilder(256); GetWindowText(h, s, 256);
    RECT r; GetWindowRect(h, out r); uint pid; GetWindowThreadProcessId(h, out pid);
    return String.Format("'{0}' pid={1} visible={2} rect=({3},{4})-({5},{6}) exstyle=0x{7:X}",
      s, pid, IsWindowVisible(h), r.L, r.T, r.R, r.B, GetWindowLongPtr(h, -20));
  }
  public static List<string> ForProcess(uint target) {
    var o = new List<string>();
    EnumWindows((h, l) => { uint p; GetWindowThreadProcessId(h, out p);
      if (p == target) { var d = Describe(h); if (!d.Contains("IME") && !d.Contains("dummy")) o.Add(d); }
      return true; }, IntPtr.Zero);
    return o;
  }
  public static int[] FindDeck() {
    int[] res = null;
    EnumWindows((h, l) => { var s = new StringBuilder(256); GetWindowText(h, s, 256);
      if (s.ToString() == "Virtual Stream Deck") { RECT r; GetWindowRect(h, out r); res = new int[] { r.L, r.T, r.R, r.B, IsWindowVisible(h) ? 1 : 0 }; }
      return true; }, IntPtr.Zero);
    return res;
  }
  public static string TopAt(int x, int y) {
    var h = GetAncestor(WindowFromPoint(new POINT { X = x, Y = y }), 2);
    return Describe(h);
  }
}
"@
[DeckStatusWin]::SetProcessDpiAwarenessContext([IntPtr]-4) | Out-Null

"== Process"
$proc = Get-Process opendeck -ErrorAction SilentlyContinue
if (-not $proc) { "OpenDeck is not running." } else { $proc | Select-Object Id, StartTime, Path | Format-List | Out-String -Width 200 }

"== Screens (physical pixels)"
Add-Type -AssemblyName System.Windows.Forms, System.Drawing
[System.Windows.Forms.Screen]::AllScreens | ForEach-Object { "{0} primary={1} {2}" -f $_.DeviceName, $_.Primary, $_.Bounds }

"== Config ($env:APPDATA\opendeck\virtual_deck.json)"
Get-Content "$env:APPDATA\opendeck\virtual_deck.json" -Encoding utf8 -ErrorAction SilentlyContinue

if ($proc) {
  "== OpenDeck windows"
  foreach ($p in $proc) { [DeckStatusWin]::ForProcess([uint32]$p.Id) }
}

"== Blockers"
$osk = Get-Process osk -ErrorAction SilentlyContinue
if ($osk) { "On-screen keyboard (osk.exe) is open: it puts an 'Input Occlusion Window' over the monitor it is on." }
$elgato = Get-Process StreamDeck -ErrorAction SilentlyContinue
if ($elgato) { "Elgato Stream Deck software is running (StreamDeck.exe)." }

$deck = [DeckStatusWin]::FindDeck()
if (-not $deck) { "== Deck window: NOT FOUND"; return }
"== Deck window: rect=({0},{1})-({2},{3}) visible={4}" -f ([object[]]$deck)
$cx = [int](($deck[0] + $deck[2]) / 2); $cy = [int](($deck[1] + $deck[3]) / 2)
"Top window at deck centre ($cx,$cy): " + [DeckStatusWin]::TopAt($cx, $cy)

$w = $deck[2] - $deck[0]; $h = $deck[3] - $deck[1]
if ($w -gt 0 -and $h -gt 0) {
  $bmp = New-Object System.Drawing.Bitmap $w, $h
  [System.Drawing.Graphics]::FromImage($bmp).CopyFromScreen($deck[0], $deck[1], 0, 0, $bmp.Size)
  $counts = @{}
  for ($y = 0; $y -lt $h; $y += 8) { for ($x = 0; $x -lt $w; $x += 8) { $c = $bmp.GetPixel($x, $y); $k = "$($c.R),$($c.G),$($c.B)"; $counts[$k]++ } }
  "Most common colours in the deck area (empty keys are about 23,23,23 on black):"
  $counts.GetEnumerator() | Sort-Object Value -Descending | Select-Object -First 5 | ForEach-Object { "  rgb($($_.Key)) x$($_.Value)" }
  $file = Join-Path $OutDir "deck-status.png"
  $bmp.Save($file)
  "Screenshot: $file"
}
