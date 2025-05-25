using System.IO;
using System.Text.Json;

namespace MicMuteOverlay
{
    public class Config
    {
        public string Hotkey { get; set; } = "Ctrl+Shift+M";
        public string OverlayText { get; set; } = "Mic Muted";
        public string MuteSound { get; set; } = "Sounds/mute.wav";
        public string UnmuteSound { get; set; } = "Sounds/unmute.wav";
        public int FontSize { get; set; } = 20;
        public string ForeColor { get; set; } = "Red";

        private const string ConfigPath = "config.json";

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
        }
    }
}