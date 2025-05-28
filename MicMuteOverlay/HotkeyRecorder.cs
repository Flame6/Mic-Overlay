using System;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text;
using System.Windows.Forms;

namespace MicMuteOverlay
{
    public class HotkeyRecorder : IDisposable
    {
        private readonly HashSet<Keys> _pressedKeys = new HashSet<Keys>();
        private readonly HashSet<Keys> _allPressedKeys = new HashSet<Keys>();
        private readonly LowLevelKeyboardProc _proc;
        private IntPtr _hookID = IntPtr.Zero;
        private bool _recording = false;

        public bool IsRecording => _recording;
        public event EventHandler<string>? HotkeyRecorded;

        public HotkeyRecorder()
        {
            _proc = HookCallback;
        }

        public void StartRecording()
        {
            if (_recording) return;

            _recording = true;
            _pressedKeys.Clear();
            _allPressedKeys.Clear();
            _hookID = SetHook(_proc);
        }

        public void StopRecording()
        {
            if (!_recording) return;

            _recording = false;
            if (_hookID != IntPtr.Zero)
            {
                UnhookWindowsHookEx(_hookID);
                _hookID = IntPtr.Zero;
            }
        }

        private IntPtr SetHook(LowLevelKeyboardProc proc)
        {
            using var curProcess = System.Diagnostics.Process.GetCurrentProcess();
            using var curModule = curProcess.MainModule;
            if (curModule?.ModuleName != null)
            {
                return SetWindowsHookEx(WH_KEYBOARD_LL, proc,
                    GetModuleHandle(curModule.ModuleName), 0);
            }
            return IntPtr.Zero;
        }

        private IntPtr HookCallback(int nCode, IntPtr wParam, IntPtr lParam)
        {
            if (nCode >= 0 && _recording)
            {
                bool keyDown = wParam == (IntPtr)WM_KEYDOWN || wParam == (IntPtr)WM_SYSKEYDOWN;
                bool keyUp = wParam == (IntPtr)WM_KEYUP || wParam == (IntPtr)WM_SYSKEYUP;

                if (keyDown || keyUp)
                {
                    int vkCode = Marshal.ReadInt32(lParam);
                    Keys key = (Keys)vkCode;

                    if (keyDown)
                    {
                        _pressedKeys.Add(key);
                        _allPressedKeys.Add(key);
                    }
                    else if (keyUp)
                    {
                        _pressedKeys.Remove(key);

                        // If all keys are released and we had keys pressed, fire the event
                        if (_pressedKeys.Count == 0 && _allPressedKeys.Count > 0)
                        {
                            var hotkeyString = BuildHotkeyString(_allPressedKeys);
                            if (!string.IsNullOrEmpty(hotkeyString))
                            {
                                HotkeyRecorded?.Invoke(this, hotkeyString);
                                StopRecording();
                            }
                        }
                    }
                }
            }

            return CallNextHookEx(_hookID, nCode, wParam, lParam);
        }

        private string BuildHotkeyString(HashSet<Keys> keys)
        {
            if (keys.Count == 0) return "";

            var parts = new List<string>();
            var sortedKeys = keys.OrderBy(GetKeyPriority).ToList();

            foreach (var key in sortedKeys)
            {
                string keyName = GetKeyName(key);
                if (!string.IsNullOrEmpty(keyName))
                {
                    parts.Add(keyName);
                }
            }

            return string.Join("+", parts);
        }

        private int GetKeyPriority(Keys key)
        {
            // Order: Ctrl, Alt, Shift, Win, then other keys
            return key switch
            {
                Keys.LControlKey or Keys.RControlKey or Keys.ControlKey => 1,
                Keys.LMenu or Keys.RMenu or Keys.Menu => 2,
                Keys.LShiftKey or Keys.RShiftKey or Keys.ShiftKey => 3,
                Keys.LWin or Keys.RWin => 4,
                _ => 5
            };
        }

        private string GetKeyName(Keys key)
        {
            return key switch
            {
                Keys.LControlKey or Keys.RControlKey or Keys.ControlKey => "Ctrl",
                Keys.LMenu or Keys.RMenu or Keys.Menu => "Alt",
                Keys.LShiftKey or Keys.RShiftKey or Keys.ShiftKey => "Shift",
                Keys.LWin or Keys.RWin => "Win",
                Keys.Space => "Space",
                Keys.Enter => "Enter",
                Keys.Tab => "Tab",
                Keys.Escape => "Esc",
                Keys.Back => "Backspace",
                Keys.Delete => "Delete",
                Keys.Insert => "Insert",
                Keys.Home => "Home",
                Keys.End => "End",
                Keys.PageUp => "PageUp",
                Keys.PageDown => "PageDown",
                Keys.Up => "Up",
                Keys.Down => "Down",
                Keys.Left => "Left",
                Keys.Right => "Right",
                Keys.F1 => "F1",
                Keys.F2 => "F2",
                Keys.F3 => "F3",
                Keys.F4 => "F4",
                Keys.F5 => "F5",
                Keys.F6 => "F6",
                Keys.F7 => "F7",
                Keys.F8 => "F8",
                Keys.F9 => "F9",
                Keys.F10 => "F10",
                Keys.F11 => "F11",
                Keys.F12 => "F12",
                Keys.D0 => "0",
                Keys.D1 => "1",
                Keys.D2 => "2",
                Keys.D3 => "3",
                Keys.D4 => "4",
                Keys.D5 => "5",
                Keys.D6 => "6",
                Keys.D7 => "7",
                Keys.D8 => "8",
                Keys.D9 => "9",
                Keys.NumPad0 => "Num0",
                Keys.NumPad1 => "Num1",
                Keys.NumPad2 => "Num2",
                Keys.NumPad3 => "Num3",
                Keys.NumPad4 => "Num4",
                Keys.NumPad5 => "Num5",
                Keys.NumPad6 => "Num6",
                Keys.NumPad7 => "Num7",
                Keys.NumPad8 => "Num8",
                Keys.NumPad9 => "Num9",
                Keys.Multiply => "Num*",
                Keys.Add => "Num+",
                Keys.Subtract => "Num-",
                Keys.Divide => "Num/",
                Keys.Decimal => "Num.",
                _ when key >= Keys.A && key <= Keys.Z => key.ToString(),
                _ => ""
            };
        }

        public void Dispose()
        {
            StopRecording();
        }

        // Windows API declarations
        private const int WH_KEYBOARD_LL = 13;
        private const int WM_KEYDOWN = 0x0100;
        private const int WM_KEYUP = 0x0101;
        private const int WM_SYSKEYDOWN = 0x0104;
        private const int WM_SYSKEYUP = 0x0105;

        private delegate IntPtr LowLevelKeyboardProc(int nCode, IntPtr wParam, IntPtr lParam);

        [DllImport("user32.dll", CharSet = CharSet.Auto, SetLastError = true)]
        private static extern IntPtr SetWindowsHookEx(int idHook, LowLevelKeyboardProc lpfn, IntPtr hMod, uint dwThreadId);

        [DllImport("user32.dll", CharSet = CharSet.Auto, SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool UnhookWindowsHookEx(IntPtr hhk);

        [DllImport("user32.dll", CharSet = CharSet.Auto, SetLastError = true)]
        private static extern IntPtr CallNextHookEx(IntPtr hhk, int nCode, IntPtr wParam, IntPtr lParam);

        [DllImport("kernel32.dll", CharSet = CharSet.Auto, SetLastError = true)]
        private static extern IntPtr GetModuleHandle(string lpModuleName);
    }
}