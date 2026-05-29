using System;
using System.Drawing;
using System.IO;
using System.Windows.Forms;

namespace MicMuteOverlay.Forms
{
    public class SettingsForm : Form
    {
        private readonly Config _config;
        private readonly HotkeyManager _hotkeyManager;
        private readonly OverlayForm _overlay;
        private readonly MicController _micController;
        private HotkeyRecorder? _hotkeyRecorder;

        // Controls
        private TextBox _hotkeyBox;
        private Button _recordHotkeyButton;
        private TextBox _textBox;
        private TextBox _muteBox;
        private TextBox _unmuteBox;
        private NumericUpDown _fontSize;
        private ComboBox _colorBox;
        private ComboBox _outlineColorBox;
        private NumericUpDown _outlineThickness;
        private ComboBox _microphoneBox;
        private CheckBox _startWithWindowsBox;
        private ComboBox _displayModeBox;
        private CheckBox _clickThroughBox;

        public SettingsForm(Config config, HotkeyManager hotkeyManager, OverlayForm overlay, MicController micController)
        {
            _config = config;
            _hotkeyManager = hotkeyManager;
            _overlay = overlay;
            _micController = micController;

            InitializeComponent();

            // Force proper layout and refresh
            this.Load += SettingsForm_Load;
            this.Shown += SettingsForm_Shown;
        }

        private void SettingsForm_Load(object? sender, EventArgs e)
        {
            // Force layout calculation
            this.PerformLayout();
            this.Refresh();
            this.Update();
        }

        private void SettingsForm_Shown(object? sender, EventArgs e)
        {
            // Additional refresh after the form is fully shown
            this.Invalidate(true);
            this.Update();

            // Force repaint of all child controls
            foreach (Control control in this.Controls)
            {
                control.Invalidate();
                control.Update();
            }
        }

        private void InitializeComponent()
        {
            // Suspend layout during control creation
            this.SuspendLayout();

            // Form setup
            Text = "MicMuteOverlay Settings";
            Size = new Size(480, 600);
            FormBorderStyle = FormBorderStyle.FixedDialog;
            MaximizeBox = false;
            MinimizeBox = false;
            StartPosition = FormStartPosition.CenterScreen;
            Font = new Font("Segoe UI", 9F);
            AutoScaleMode = AutoScaleMode.Font;
            BackColor = SystemColors.Control;

            int y = 20;
            int labelX = 20;
            int controlX = 160;
            int controlWidth = 280;
            int rowSpacing = 35;

            // Hotkey
            var hotkeyLabel = new Label();
            hotkeyLabel.Text = "Hotkey:";
            hotkeyLabel.Location = new Point(labelX, y + 3);
            hotkeyLabel.Size = new Size(120, 23);
            Controls.Add(hotkeyLabel);

            _hotkeyBox = new TextBox();
            _hotkeyBox.Text = _config.Hotkey;
            _hotkeyBox.Location = new Point(controlX, y);
            _hotkeyBox.Size = new Size(180, 23);
            _hotkeyBox.ReadOnly = true;
            Controls.Add(_hotkeyBox);

            _recordHotkeyButton = new Button();
            _recordHotkeyButton.Text = "Record";
            _recordHotkeyButton.Location = new Point(controlX + 190, y);
            _recordHotkeyButton.Size = new Size(70, 23);
            _recordHotkeyButton.Click += RecordHotkey_Click;
            Controls.Add(_recordHotkeyButton);

            y += rowSpacing;

            // Overlay Text
            var textLabel = new Label();
            textLabel.Text = "Overlay Text:";
            textLabel.Location = new Point(labelX, y + 3);
            textLabel.Size = new Size(120, 23);
            Controls.Add(textLabel);

            _textBox = new TextBox();
            _textBox.Text = _config.OverlayText;
            _textBox.Location = new Point(controlX, y);
            _textBox.Size = new Size(controlWidth, 23);
            Controls.Add(_textBox);

            y += rowSpacing;

            // Mute Sound
            var muteLabel = new Label();
            muteLabel.Text = "Mute Sound:";
            muteLabel.Location = new Point(labelX, y + 3);
            muteLabel.Size = new Size(120, 23);
            Controls.Add(muteLabel);

            _muteBox = new TextBox();
            _muteBox.Text = _config.MuteSound;
            _muteBox.Location = new Point(controlX, y);
            _muteBox.Size = new Size(controlWidth, 23);
            Controls.Add(_muteBox);

            y += rowSpacing;

            // Unmute Sound
            var unmuteLabel = new Label();
            unmuteLabel.Text = "Unmute Sound:";
            unmuteLabel.Location = new Point(labelX, y + 3);
            unmuteLabel.Size = new Size(120, 23);
            Controls.Add(unmuteLabel);

            _unmuteBox = new TextBox();
            _unmuteBox.Text = _config.UnmuteSound;
            _unmuteBox.Location = new Point(controlX, y);
            _unmuteBox.Size = new Size(controlWidth, 23);
            Controls.Add(_unmuteBox);

            y += rowSpacing;

            // Font Size
            var fontLabel = new Label();
            fontLabel.Text = "Font Size:";
            fontLabel.Location = new Point(labelX, y + 3);
            fontLabel.Size = new Size(120, 23);
            Controls.Add(fontLabel);

            _fontSize = new NumericUpDown();
            _fontSize.Minimum = 8;
            _fontSize.Maximum = 72;
            _fontSize.Value = _config.FontSize;
            _fontSize.Location = new Point(controlX, y);
            _fontSize.Size = new Size(80, 23);
            Controls.Add(_fontSize);

            y += rowSpacing;

            // Text Color
            var colorLabel = new Label();
            colorLabel.Text = "Text Color:";
            colorLabel.Location = new Point(labelX, y + 3);
            colorLabel.Size = new Size(120, 23);
            Controls.Add(colorLabel);

            _colorBox = new ComboBox();
            _colorBox.Location = new Point(controlX, y);
            _colorBox.Size = new Size(120, 23);
            _colorBox.DropDownStyle = ComboBoxStyle.DropDownList;
            PopulateColorComboBox(_colorBox, _config.ForeColor);
            Controls.Add(_colorBox);

            y += rowSpacing;

            // Outline Color  
            var outlineColorLabel = new Label();
            outlineColorLabel.Text = "Outline Color:";
            outlineColorLabel.Location = new Point(labelX, y + 3);
            outlineColorLabel.Size = new Size(120, 23);
            Controls.Add(outlineColorLabel);

            _outlineColorBox = new ComboBox();
            _outlineColorBox.Location = new Point(controlX, y);
            _outlineColorBox.Size = new Size(120, 23);
            _outlineColorBox.DropDownStyle = ComboBoxStyle.DropDownList;
            PopulateColorComboBox(_outlineColorBox, _config.OutlineColor);
            Controls.Add(_outlineColorBox);

            y += rowSpacing;

            // Outline Thickness
            var outlineThicknessLabel = new Label();
            outlineThicknessLabel.Text = "Outline Thickness:";
            outlineThicknessLabel.Location = new Point(labelX, y + 3);
            outlineThicknessLabel.Size = new Size(120, 23);
            Controls.Add(outlineThicknessLabel);

            _outlineThickness = new NumericUpDown();
            _outlineThickness.Minimum = 0;
            _outlineThickness.Maximum = 10;
            _outlineThickness.Value = _config.OutlineThickness;
            _outlineThickness.Location = new Point(controlX, y);
            _outlineThickness.Size = new Size(80, 23);
            Controls.Add(_outlineThickness);

            y += rowSpacing;

            // Microphone
            var micLabel = new Label();
            micLabel.Text = "Microphone:";
            micLabel.Location = new Point(labelX, y + 3);
            micLabel.Size = new Size(120, 23);
            Controls.Add(micLabel);

            _microphoneBox = new ComboBox();
            _microphoneBox.Location = new Point(controlX, y);
            _microphoneBox.Size = new Size(controlWidth, 23);
            _microphoneBox.DropDownStyle = ComboBoxStyle.DropDownList;
            LoadMicrophones(_config.SelectedMicrophoneId);
            Controls.Add(_microphoneBox);

            y += rowSpacing;

            // Show Overlay
            var displayModeLabel = new Label();
            displayModeLabel.Text = "Show Overlay:";
            displayModeLabel.Location = new Point(labelX, y + 3);
            displayModeLabel.Size = new Size(120, 23);
            Controls.Add(displayModeLabel);

            _displayModeBox = new ComboBox();
            _displayModeBox.Location = new Point(controlX, y);
            _displayModeBox.Size = new Size(150, 23);
            _displayModeBox.DropDownStyle = ComboBoxStyle.DropDownList;
            PopulateDisplayModeComboBox();
            Controls.Add(_displayModeBox);

            y += rowSpacing;

            // Start with Windows
            _startWithWindowsBox = new CheckBox();
            _startWithWindowsBox.Text = "Start with Windows";
            _startWithWindowsBox.Location = new Point(labelX, y);
            _startWithWindowsBox.Size = new Size(200, 25);
            _startWithWindowsBox.Checked = _config.IsSetToStartWithWindows();
            Controls.Add(_startWithWindowsBox);

            y += 30;

            // Click-through mode
            _clickThroughBox = new CheckBox();
            _clickThroughBox.Text = "Click-through mode (overlay ignores mouse clicks)";
            _clickThroughBox.Location = new Point(labelX, y);
            _clickThroughBox.Size = new Size(400, 25);
            _clickThroughBox.Checked = _config.ClickThroughMode;
            Controls.Add(_clickThroughBox);

            y += 40;

            // Buttons
            var saveButton = new Button();
            saveButton.Text = "Save & Close";
            saveButton.Location = new Point(labelX, y);
            saveButton.Size = new Size(100, 30);
            saveButton.Click += Save_Click;
            Controls.Add(saveButton);

            var cancelButton = new Button();
            cancelButton.Text = "Cancel";
            cancelButton.Location = new Point(labelX + 110, y);
            cancelButton.Size = new Size(100, 30);
            cancelButton.Click += (s, e) => Close();
            Controls.Add(cancelButton);

            var previewButton = new Button();
            previewButton.Text = "Preview";
            previewButton.Location = new Point(labelX + 220, y);
            previewButton.Size = new Size(100, 30);
            previewButton.Click += Preview_Click;
            Controls.Add(previewButton);

            // Resume layout and force refresh
            this.ResumeLayout(true);
            this.PerformLayout();
        }

        private void PopulateColorComboBox(ComboBox comboBox, string selectedColor)
        {
            var colors = new[] {
                "Red", "White", "Black", "Blue", "Green", "Yellow",
                "Orange", "Purple", "Pink", "Cyan", "Magenta", "Gray",
                "DarkRed", "DarkBlue", "DarkGreen", "Navy", "Maroon"
            };

            comboBox.Items.AddRange(colors);
            comboBox.Text = selectedColor;
        }

        private void PopulateDisplayModeComboBox()
        {
            _displayModeBox.Items.AddRange(new[] {
                "When Muted", "When Unmuted", "Always", "Never"
            });

            _displayModeBox.SelectedIndex = (int)_config.DisplayMode;
        }

        private void LoadMicrophones(string selectedId)
        {
            _microphoneBox.Items.Clear();
            var microphones = _micController.GetAvailableMicrophones();

            foreach (var mic in microphones)
            {
                _microphoneBox.Items.Add(mic);
            }

            if (!string.IsNullOrEmpty(selectedId))
            {
                for (int i = 0; i < _microphoneBox.Items.Count; i++)
                {
                    if (((MicrophoneInfo)_microphoneBox.Items[i]).Id == selectedId)
                    {
                        _microphoneBox.SelectedIndex = i;
                        break;
                    }
                }
            }

            if (_microphoneBox.SelectedIndex == -1 && _microphoneBox.Items.Count > 0)
            {
                _microphoneBox.SelectedIndex = 0;
            }
        }

        private void RecordHotkey_Click(object? sender, EventArgs e)
        {
            if (_hotkeyRecorder?.IsRecording == true)
            {
                _hotkeyRecorder.StopRecording();
                _recordHotkeyButton.Text = "Record";
                _hotkeyBox.BackColor = SystemColors.Window;
                return;
            }

            _hotkeyRecorder?.Dispose();
            _hotkeyRecorder = new HotkeyRecorder();
            _hotkeyRecorder.HotkeyRecorded += (s, hotkey) => {
                if (InvokeRequired)
                {
                    Invoke(() => {
                        _hotkeyBox.Text = hotkey;
                        _recordHotkeyButton.Text = "Record";
                        _hotkeyBox.BackColor = SystemColors.Window;
                    });
                }
                else
                {
                    _hotkeyBox.Text = hotkey;
                    _recordHotkeyButton.Text = "Record";
                    _hotkeyBox.BackColor = SystemColors.Window;
                }
            };

            _hotkeyRecorder.StartRecording();
            _recordHotkeyButton.Text = "Stop";
            _hotkeyBox.BackColor = Color.LightYellow;
            _hotkeyBox.Text = "Press key combination...";
        }

        private void Preview_Click(object? sender, EventArgs e)
        {
            // Temporarily apply settings for preview
            var originalText = _config.OverlayText;
            var originalFontSize = _config.FontSize;
            var originalForeColor = _config.ForeColor;
            var originalOutlineColor = _config.OutlineColor;
            var originalOutlineThickness = _config.OutlineThickness;

            _config.OverlayText = _textBox.Text;
            _config.FontSize = (int)_fontSize.Value;
            _config.ForeColor = _colorBox.Text;
            _config.OutlineColor = _outlineColorBox.Text;
            _config.OutlineThickness = (int)_outlineThickness.Value;

            _overlay.ReloadConfig();
            _overlay.UpdateStatus(true); // Show as muted for preview

            // Restore original settings after 3 seconds
            var timer = new Timer { Interval = 3000, Enabled = true };
            timer.Tick += (s, args) => {
                timer.Dispose();
                _config.OverlayText = originalText;
                _config.FontSize = originalFontSize;
                _config.ForeColor = originalForeColor;
                _config.OutlineColor = originalOutlineColor;
                _config.OutlineThickness = originalOutlineThickness;
                _overlay.ReloadConfig();
                _overlay.UpdateStatus(_micController.IsMuted);
            };
        }

        private void Save_Click(object? sender, EventArgs e)
        {
            _config.Hotkey = _hotkeyBox.Text;
            _config.OverlayText = _textBox.Text;
            _config.MuteSound = _muteBox.Text;
            _config.UnmuteSound = _unmuteBox.Text;
            _config.FontSize = (int)_fontSize.Value;
            _config.ForeColor = _colorBox.Text;
            _config.OutlineColor = _outlineColorBox.Text;
            _config.OutlineThickness = (int)_outlineThickness.Value;
            _config.StartWithWindows = _startWithWindowsBox.Checked;
            _config.DisplayMode = (OverlayDisplayMode)_displayModeBox.SelectedIndex;
            _config.ClickThroughMode = _clickThroughBox.Checked;

            if (_microphoneBox.SelectedItem is MicrophoneInfo selectedMic)
            {
                _config.SelectedMicrophoneId = selectedMic.Id;
                _micController.SetMicrophone(selectedMic.Id);
            }

            _hotkeyManager.UpdateHotkey(_config.Hotkey);
            _config.Save();
            _overlay.ReloadConfig();
            Close();
        }

        protected override void OnFormClosed(FormClosedEventArgs e)
        {
            _hotkeyRecorder?.Dispose();
            base.OnFormClosed(e);
        }
    }
}