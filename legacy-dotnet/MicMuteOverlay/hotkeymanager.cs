using System;
using System.Runtime.InteropServices;
using System.Windows.Forms;

namespace MicMuteOverlay
{
    public class HotkeyManager : IDisposable
    {
        private int _id;
        private Keys _key;
        private KeyModifiers _modifiers;
        private HotkeyMessageFilter? _filter;

        public event EventHandler? HotkeyPressed;

        public HotkeyManager(string hotkey)
        {
            (_modifiers, _key) = ParseHotkey(hotkey);
            _id = GetHashCode();
            RegisterHotKey(IntPtr.Zero, _id, (uint)_modifiers, (uint)_key);
            _filter = new HotkeyMessageFilter(_id, OnHotkey);
            Application.AddMessageFilter(_filter);
        }

        public void UpdateHotkey(string hotkey)
        {
            if (_filter != null) Application.RemoveMessageFilter(_filter);
            UnregisterHotKey(IntPtr.Zero, _id);
            (_modifiers, _key) = ParseHotkey(hotkey);
            _id = GetHashCode();
            RegisterHotKey(IntPtr.Zero, _id, (uint)_modifiers, (uint)_key);
            _filter = new HotkeyMessageFilter(_id, OnHotkey);
            Application.AddMessageFilter(_filter);
        }

        private void OnHotkey()
        {
            HotkeyPressed?.Invoke(this, EventArgs.Empty);
        }

        public void Dispose()
        {
            if (_filter != null) Application.RemoveMessageFilter(_filter);
            UnregisterHotKey(IntPtr.Zero, _id);
        }

        private static (KeyModifiers mods, Keys key) ParseHotkey(string text)
        {
            KeyModifiers mods = 0;
            Keys key = Keys.None;
            foreach (var part in text.Split('+', StringSplitOptions.RemoveEmptyEntries))
            {
                var p = part.Trim();
                switch (p.ToLowerInvariant())
                {
                    case "ctrl": mods |= KeyModifiers.Control; break;
                    case "shift": mods |= KeyModifiers.Shift; break;
                    case "alt": mods |= KeyModifiers.Alt; break;
                    case "win": mods |= KeyModifiers.Win; break;
                    default:
                        if (Enum.TryParse<Keys>(p, true, out var k)) key = k;
                        break;
                }
            }
            return (mods, key);
        }

        private class HotkeyMessageFilter : IMessageFilter
        {
            private readonly int _id;
            private readonly Action _action;
            public HotkeyMessageFilter(int id, Action action)
            {
                _id = id; _action = action;
            }
            public bool PreFilterMessage(ref Message m)
            {
                if (m.Msg == WM_HOTKEY && m.WParam.ToInt32() == _id)
                {
                    _action();
                    return true;
                }
                return false;
            }
        }

        private const int WM_HOTKEY = 0x0312;

        [DllImport("user32.dll")]
        private static extern bool RegisterHotKey(IntPtr hWnd, int id, uint fsModifiers, uint vk);

        [DllImport("user32.dll")]
        private static extern bool UnregisterHotKey(IntPtr hWnd, int id);

        [Flags]
        private enum KeyModifiers
        {
            Alt = 1,
            Control = 2,
            Shift = 4,
            Win = 8
        }
    }
}