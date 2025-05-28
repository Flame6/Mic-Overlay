using System.IO;
using System.Text.Json;
using Microsoft.Win32;

namespace MicMuteOverlay
{
    public enum OverlayDisplayMode
    {
        WhenMuted = 0,
        WhenUnmuted = 1,
        Always = 2,
        Never = 3
    }

    public class Config
    {
        public string Hotkey { get; set; } = "Ctrl+Shift+M";
        public string OverlayText { get; set; } = "Mic Muted";
        public string MuteSound { get; set; } = "Sounds/mute.wav";
        public string UnmuteSound { get; set; } = "Sounds/unmute.wav";
        public int FontSize { get; set; } = 20;
        public string ForeColor { get; set; } = "Red";
        public string SelectedMicrophoneId { get; set; } = "";

        // New properties
        public bool StartWithWindows { get; set; } = false;
        public string OutlineColor { get; set; } = "Black";
        public int OutlineThickness { get; set; } = 2;
        public OverlayDisplayMode DisplayMode { get; set; } = OverlayDisplayMode.WhenMuted;
        public bool ClickThroughMode { get; set; } = false;

        private const string ConfigPath = "config.json";
        private const string RegistryKey = @"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";
        private const string AppName = "MicMuteOverlay";

        public static Config Load()
        {
            if (File.Exists(ConfigPath))
            {
                var json = File.ReadAllText(ConfigPath);
                return JsonSerializer.Deserialize<Config>(json) ?? new Config();
            }
            return new Config();
        }

        public void Save()
        {
            var json = JsonSerializer.Serialize(this, new JsonSerializerOptions { WriteIndented = true });
            File.WriteAllText(ConfigPath, json);

            // Handle Windows startup setting
            SetWindowsStartup(StartWithWindows);
        }

        private void SetWindowsStartup(bool enable)
        {
            try
            {
                using var key = Registry.CurrentUser.OpenSubKey(RegistryKey, true);
                if (key != null)
                {
                    if (enable)
                    {
                        var exePath = System.Reflection.Assembly.GetExecutingAssembly().Location;
                        if (exePath.EndsWith(".dll"))
                        {
                            // For .NET 6 single-file deployment, get the actual exe path
                            exePath = System.Diagnostics.Process.GetCurrentProcess().MainModule?.FileName ?? exePath;
                        }
                        key.SetValue(AppName, $"\"{exePath}\"");
                    }
                    else
                    {
                        key.DeleteValue(AppName, false);
                    }
                }
            }
            catch
            {
                // Ignore registry errors
            }
        }

        public bool IsSetToStartWithWindows()
        {
            try
            {
                using var key = Registry.CurrentUser.OpenSubKey(RegistryKey, false);
                return key?.GetValue(AppName) != null;
            }
            catch
            {
                return false;
            }
        }
    }
}