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
            using var controller = new MicController(config.SelectedMicrophoneId);
            Hotkeys = new HotkeyManager(config.Hotkey);

            var overlay = new OverlayForm(controller, config);

            // Create tray manager
            using var trayManager = new TrayManager(config, Hotkeys, overlay, controller);

            Hotkeys.HotkeyPressed += (s, e) =>
            {
                controller.ToggleMute();
                overlay.UpdateStatus(controller.IsMuted);
            };

            // Set initial overlay visibility based on display mode and current mute state
            overlay.UpdateStatus(controller.IsMuted);

            // Hide the overlay from taskbar since we now have a tray icon
            overlay.WindowState = FormWindowState.Normal;
            overlay.ShowInTaskbar = false;

            Application.Run(overlay);

            config.Save();
            Hotkeys?.Dispose();
        }
    }
}