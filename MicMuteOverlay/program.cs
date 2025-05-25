using System;
using System.Windows.Forms;
using MicMuteOverlay.Forms;

namespace MicMuteOverlay
{
    static class Program
    {
        public static HotkeyManager? Hotkeys { get; private set; }
        [STAThread]
        static void Main()
        {
            Application.SetHighDpiMode(HighDpiMode.SystemAware);
            Application.EnableVisualStyles();
            Application.SetCompatibleTextRenderingDefault(false);

            var config = Config.Load();
            using var controller = new MicController();
            Hotkeys = new HotkeyManager(config.Hotkey);

            var overlay = new OverlayForm(controller, config);

            Hotkeys.HotkeyPressed += (s, e) =>
            {
                controller.ToggleMute();
                overlay.UpdateStatus(controller.IsMuted);
            };

            Application.Run(overlay);
            config.Save();
            Hotkeys.Dispose();
        }
    }
}