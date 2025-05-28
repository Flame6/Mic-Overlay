using System;
using System.Drawing;
using System.Windows.Forms;

namespace MicMuteOverlay.Forms
{
    public class SettingsForm : Form
    {
        private readonly Config _config;
        private readonly HotkeyManager _hotkeyManager;
        private readonly OverlayForm _overlay;
        private readonly TextBox _hotkeyBox;
        private readonly TextBox _textBox;
        private readonly TextBox _muteBox;
        private readonly TextBox _unmuteBox;
        private readonly NumericUpDown _fontSize;
        private readonly TextBox _colorBox;

        public SettingsForm(Config config, HotkeyManager hotkeyManager, OverlayForm overlay)
        {
            _config = config;
            _hotkeyManager = hotkeyManager;
            _overlay = overlay;

            Text = "Settings";
            Size = new Size(350, 260);

            var hotkeyLabel = new Label { Text = "Hotkey:", Location = new Point(10, 10) };
            _hotkeyBox = new TextBox { Text = config.Hotkey, Location = new Point(120, 10), Width = 200 };

            var textLabel = new Label { Text = "Overlay Text:", Location = new Point(10, 40) };
            _textBox = new TextBox { Text = config.OverlayText, Location = new Point(120, 40), Width = 200 };

            var muteLabel = new Label { Text = "Mute Sound:", Location = new Point(10, 70) };
            _muteBox = new TextBox { Text = config.MuteSound, Location = new Point(120, 70), Width = 200 };

            var unmuteLabel = new Label { Text = "Unmute Sound:", Location = new Point(10, 100) };
            _unmuteBox = new TextBox { Text = config.UnmuteSound, Location = new Point(120, 100), Width = 200 };

            var fontLabel = new Label { Text = "Font Size:", Location = new Point(10, 130) };
            _fontSize = new NumericUpDown { Minimum = 8, Maximum = 72, Value = config.FontSize, Location = new Point(120, 130), Width = 60 };

            var colorLabel = new Label { Text = "Text Color:", Location = new Point(10, 160) };
            _colorBox = new TextBox { Text = config.ForeColor, Location = new Point(120, 160), Width = 100 };

            var save = new Button { Text = "Save & Close", Location = new Point(10, 190), Width = 310 };
            save.Click += Save_Click;

            Controls.Add(hotkeyLabel);
            Controls.Add(_hotkeyBox);
            Controls.Add(textLabel);
            Controls.Add(_textBox);
            Controls.Add(muteLabel);
            Controls.Add(_muteBox);
            Controls.Add(unmuteLabel);
            Controls.Add(_unmuteBox);
            Controls.Add(fontLabel);
            Controls.Add(_fontSize);
            Controls.Add(colorLabel);
            Controls.Add(_colorBox);
            Controls.Add(save);
        }

        private void Save_Click(object? sender, EventArgs e)
        {
            _config.Hotkey = _hotkeyBox.Text;
            _config.OverlayText = _textBox.Text;
            _config.MuteSound = _muteBox.Text;
            _config.UnmuteSound = _unmuteBox.Text;
            _config.FontSize = (int)_fontSize.Value;
            _config.ForeColor = _colorBox.Text;

            _hotkeyManager.UpdateHotkey(_config.Hotkey);
            _config.Save();
            _overlay.ReloadConfig();
            Close(); // 👈 this is what was probably missing
        }
    }
}
