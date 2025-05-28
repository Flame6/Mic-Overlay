using MicMuteOverlay.Forms;
using System;
using System.Drawing;
using System.IO;
using System.Reflection;
using System.Windows.Forms;

namespace MicMuteOverlay
{
    public class TrayManager : IDisposable
    {
        private readonly NotifyIcon _notifyIcon;
        private readonly Config _config;
        private readonly HotkeyManager _hotkeyManager;
        private readonly OverlayForm _overlay;

        public TrayManager(Config config, HotkeyManager hotkeyManager, OverlayForm overlay)
        {
            _config = config;
            _hotkeyManager = hotkeyManager;
            _overlay = overlay;

            _notifyIcon = new NotifyIcon
            {
                Icon = LoadEmbeddedIcon(),
                Text = "MicMuteOverlay",
                Visible = true
            };

            CreateContextMenu();
        }

        private Icon LoadEmbeddedIcon()
        {
            try
            {
                // Load the embedded icon
                var assembly = Assembly.GetExecutingAssembly();
                var resourceName = "MicMuteOverlay.icon.ico";

                using var stream = assembly.GetManifestResourceStream(resourceName);
                if (stream != null)
                {
                    return new Icon(stream);
                }
            }
            catch
            {
                // Fallback to default icon if loading fails
            }

            // Create a simple default icon if the embedded one fails to load
            return SystemIcons.Application;
        }

        private void CreateContextMenu()
        {
            var contextMenu = new ContextMenuStrip();

            // Settings menu item
            var settingsItem = new ToolStripMenuItem("Settings");
            settingsItem.Click += (s, e) => ShowSettings();
            contextMenu.Items.Add(settingsItem);

            // Reset overlay position
            var resetItem = new ToolStripMenuItem("Reset Overlay Position");
            resetItem.Click += (s, e) => ResetOverlayPosition();
            contextMenu.Items.Add(resetItem);

            // Separator
            contextMenu.Items.Add(new ToolStripSeparator());

            // Exit menu item
            var exitItem = new ToolStripMenuItem("Exit");
            exitItem.Click += (s, e) => ExitApplication();
            contextMenu.Items.Add(exitItem);

            _notifyIcon.ContextMenuStrip = contextMenu;
        }

        private void ShowSettings()
        {
            var settingsForm = new Forms.SettingsForm(_config, _hotkeyManager, _overlay);
            settingsForm.Show();
            settingsForm.Activate();
        }

        private void ResetOverlayPosition()
        {
            _overlay.Location = new Point(
    (Screen.PrimaryScreen.WorkingArea.Width - _overlay.Width) / 2,
    (Screen.PrimaryScreen.WorkingArea.Height - _overlay.Height) / 2
);
        }

        private void ExitApplication()
        {
            Application.Exit();
        }

        public void Dispose()
        {
            _notifyIcon?.Dispose();
        }
    }
}