using System;
using System.Drawing;
using System.Media;
using System.Windows.Forms;

namespace MicMuteOverlay.Forms
{
    public class OverlayForm : Form
    {
        private readonly MicController _controller;
        private readonly Config _config;
        private readonly Label _label;
        private readonly SoundPlayer _mutePlayer;
        private readonly SoundPlayer _unmutePlayer;

        public OverlayForm(MicController controller, Config config)
        {
            _controller = controller;
            _config = config;
            _label = new Label
            {
                AutoSize = true,
                BackColor = Color.Transparent
            };
            Controls.Add(_label);

            var menu = new ContextMenuStrip();
            var settingsItem = new ToolStripMenuItem("Settings");
            settingsItem.Click += (s, e) => new SettingsForm(_config, Program.Hotkeys!, this).Show();
            menu.Items.Add(settingsItem);
            var exitItem = new ToolStripMenuItem("Exit");
            exitItem.Click += (s, e) => Close();
            menu.Items.Add(exitItem);
            ContextMenuStrip = menu;

            _mutePlayer = new SoundPlayer(config.MuteSound);
            _unmutePlayer = new SoundPlayer(config.UnmuteSound);

            FormBorderStyle = FormBorderStyle.None;
            TopMost = true;
            ShowInTaskbar = false;
            BackColor = Color.Magenta;
            TransparencyKey = Color.Magenta;
            StartPosition = FormStartPosition.CenterScreen;
            ApplyAppearance();
            UpdateStatus(_controller.IsMuted);
            ResizeToFit();
        }

        public void UpdateStatus(bool muted)
        {
            _label.Text = muted ? _config.OverlayText : string.Empty;
            if (muted) _mutePlayer.Play(); else _unmutePlayer.Play();
            ResizeToFit();
            Invalidate();
        }

        public void ReloadConfig()
        {
            _mutePlayer.SoundLocation = _config.MuteSound;
            _unmutePlayer.SoundLocation = _config.UnmuteSound;
            ApplyAppearance();
            UpdateStatus(_controller.IsMuted);
        }

        private void ApplyAppearance()
        {
            _label.ForeColor = Color.FromName(_config.ForeColor);
            _label.Font = new Font("Segoe UI", _config.FontSize, FontStyle.Bold);
        }

        private void ResizeToFit()
        {
            Width = _label.Width + 20;
            Height = _label.Height + 20;
        }
    }
}