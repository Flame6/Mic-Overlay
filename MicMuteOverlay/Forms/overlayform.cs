using System;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.IO;
using System.Media;
using System.Runtime.InteropServices;
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

        private bool _dragging;
        private Point _dragStart;

        // Windows API for click-through functionality
        private const int GWL_EXSTYLE = -20;
        private const int WS_EX_LAYERED = 0x80000;
        private const int WS_EX_TRANSPARENT = 0x20;

        [DllImport("user32.dll", SetLastError = true)]
        private static extern int GetWindowLong(IntPtr hWnd, int nIndex);

        [DllImport("user32.dll")]
        private static extern int SetWindowLong(IntPtr hWnd, int nIndex, int dwNewLong);

        public OverlayForm(MicController controller, Config config)
        {
            _controller = controller;
            _config = config;

            _label = new Label
            {
                AutoSize = false,
                BackColor = Color.Transparent,
                ForeColor = Color.White,
                Font = new Font("Segoe UI", _config.FontSize, FontStyle.Bold),
                Cursor = Cursors.SizeAll,
                TextAlign = ContentAlignment.MiddleCenter
            };

            // Override the label's paint event to draw outline
            _label.Paint += Label_Paint;
            Controls.Add(_label);

            // Enable dragging via label only (when not in click-through mode)
            _label.MouseDown += OverlayForm_MouseDown;
            _label.MouseMove += OverlayForm_MouseMove;
            _label.MouseUp += OverlayForm_MouseUp;

            var menu = new ContextMenuStrip();
            var settingsItem = new ToolStripMenuItem("Settings");
            settingsItem.Click += (s, e) => new SettingsForm(_config, Program.Hotkeys!, this, _controller).Show();
            menu.Items.Add(settingsItem);

            var exitItem = new ToolStripMenuItem("Exit");
            exitItem.Click += (s, e) => Close();
            menu.Items.Add(exitItem);
            ContextMenuStrip = menu;

            // Initialize sound players with error handling
            _mutePlayer = new SoundPlayer();
            _unmutePlayer = new SoundPlayer();
            LoadSounds();

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

        private void LoadSounds()
        {
            try
            {
                // Try multiple possible paths for mute sound
                var mutePaths = new[]
                {
                    _config.MuteSound, // Original path from config
                    Path.Combine(Application.StartupPath, _config.MuteSound), // Relative to exe
                    Path.Combine(Directory.GetCurrentDirectory(), _config.MuteSound), // Relative to current dir
                    Path.Combine(AppDomain.CurrentDomain.BaseDirectory, _config.MuteSound), // Base directory
                    Path.GetFullPath(_config.MuteSound) // Try as absolute path
                };

                string? foundMutePath = null;
                foreach (var path in mutePaths)
                {
                    if (File.Exists(path))
                    {
                        foundMutePath = path;
                        break;
                    }
                }

                // Try multiple possible paths for unmute sound
                var unmutePaths = new[]
                {
                    _config.UnmuteSound,
                    Path.Combine(Application.StartupPath, _config.UnmuteSound),
                    Path.Combine(Directory.GetCurrentDirectory(), _config.UnmuteSound),
                    Path.Combine(AppDomain.CurrentDomain.BaseDirectory, _config.UnmuteSound),
                    Path.GetFullPath(_config.UnmuteSound)
                };

                string? foundUnmutePath = null;
                foreach (var path in unmutePaths)
                {
                    if (File.Exists(path))
                    {
                        foundUnmutePath = path;
                        break;
                    }
                }

                // Only set sound location if we found the files
                if (foundMutePath != null)
                {
                    _mutePlayer.SoundLocation = foundMutePath;
                    _mutePlayer.LoadAsync(); // Pre-load the sound
                }

                if (foundUnmutePath != null)
                {
                    _unmutePlayer.SoundLocation = foundUnmutePath;
                    _unmutePlayer.LoadAsync(); // Pre-load the sound
                }
            }
            catch (Exception ex)
            {
                // Log the error for debugging
                System.Diagnostics.Debug.WriteLine($"Sound loading error: {ex.Message}");
            }
        }

        private void Label_Paint(object? sender, PaintEventArgs e)
        {
            if (_config.OutlineThickness <= 0 || string.IsNullOrEmpty(_label.Text))
                return;

            var g = e.Graphics;
            g.SmoothingMode = SmoothingMode.AntiAlias;
            g.TextRenderingHint = System.Drawing.Text.TextRenderingHint.AntiAlias;

            var text = _label.Text;
            var font = _label.Font;
            var rect = _label.ClientRectangle;

            // Create string format for centering
            var stringFormat = new StringFormat
            {
                Alignment = StringAlignment.Center,
                LineAlignment = StringAlignment.Center
            };

            try
            {
                // Draw outline by drawing text multiple times with offset
                using var outlineBrush = new SolidBrush(Color.FromName(_config.OutlineColor));

                // Draw outline in multiple directions
                for (int x = -_config.OutlineThickness; x <= _config.OutlineThickness; x++)
                {
                    for (int y = -_config.OutlineThickness; y <= _config.OutlineThickness; y++)
                    {
                        if (x == 0 && y == 0) continue; // Skip center (main text)

                        var outlineRect = new RectangleF(rect.X + x, rect.Y + y, rect.Width, rect.Height);
                        g.DrawString(text, font, outlineBrush, outlineRect, stringFormat);
                    }
                }

                // Draw main text on top
                using var textBrush = new SolidBrush(_label.ForeColor);
                g.DrawString(text, font, textBrush, rect, stringFormat);
            }
            catch
            {
                // Fallback to default drawing if outline fails
            }
        }

        public void UpdateStatus(bool muted)
        {
            bool shouldShow = ShouldShowOverlay(muted);

            if (shouldShow)
            {
                _label.Text = _config.OverlayText;
                Show();
            }
            else
            {
                _label.Text = string.Empty;
                if (_config.DisplayMode == OverlayDisplayMode.Never)
                {
                    Hide();
                }
            }

            // Play sounds
            try
            {
                if (muted && !string.IsNullOrEmpty(_mutePlayer.SoundLocation))
                {
                    _mutePlayer.Play();
                }
                else if (!muted && !string.IsNullOrEmpty(_unmutePlayer.SoundLocation))
                {
                    _unmutePlayer.Play();
                }
            }
            catch (Exception ex)
            {
                // Log sound playback errors for debugging
                System.Diagnostics.Debug.WriteLine($"Sound playback error: {ex.Message}");
            }

            ResizeToFit();
            Invalidate();
            _label.Invalidate(); // Force label repaint for outline
        }

        private bool ShouldShowOverlay(bool muted)
        {
            return _config.DisplayMode switch
            {
                OverlayDisplayMode.WhenMuted => muted,
                OverlayDisplayMode.WhenUnmuted => !muted,
                OverlayDisplayMode.Always => true,
                OverlayDisplayMode.Never => false,
                _ => muted
            };
        }

        public void ReloadConfig()
        {
            LoadSounds(); // Reload sounds with new paths
            ApplyAppearance();
            ApplyClickThroughMode();
            UpdateStatus(_controller.IsMuted);
        }

        private void ApplyAppearance()
        {
            _label.ForeColor = Color.FromName(_config.ForeColor);
            _label.Font = new Font("Segoe UI", _config.FontSize, FontStyle.Bold);
        }

        private void ApplyClickThroughMode()
        {
            try
            {
                if (_config.ClickThroughMode)
                {
                    // Make window click-through
                    int exStyle = GetWindowLong(Handle, GWL_EXSTYLE);
                    SetWindowLong(Handle, GWL_EXSTYLE, exStyle | WS_EX_LAYERED | WS_EX_TRANSPARENT);
                }
                else
                {
                    // Make window normal
                    int exStyle = GetWindowLong(Handle, GWL_EXSTYLE);
                    SetWindowLong(Handle, GWL_EXSTYLE, exStyle & ~WS_EX_TRANSPARENT);
                }
            }
            catch
            {
                // Ignore Windows API errors
            }
        }

        protected override void OnHandleCreated(EventArgs e)
        {
            base.OnHandleCreated(e);
            ApplyClickThroughMode();
        }

        private void ResizeToFit()
        {
            if (string.IsNullOrEmpty(_label.Text))
            {
                Size = new Size(1, 1);
                return;
            }

            using var g = CreateGraphics();
            var textSize = g.MeasureString(_label.Text, _label.Font);

            // Add padding and outline thickness
            int padding = 20;
            int outlinePadding = _config.OutlineThickness * 2;

            int width = (int)textSize.Width + padding + outlinePadding;
            int height = (int)textSize.Height + padding + outlinePadding;

            Size = new Size(width, height);
            _label.Size = Size;
        }

        private void OverlayForm_MouseDown(object? sender, MouseEventArgs e)
        {
            if (_config.ClickThroughMode) return;

            _dragging = true;
            _dragStart = e.Location;
        }

        private void OverlayForm_MouseMove(object? sender, MouseEventArgs e)
        {
            if (_config.ClickThroughMode || !_dragging) return;

            Location = new Point(Location.X + e.X - _dragStart.X, Location.Y + e.Y - _dragStart.Y);
        }

        private void OverlayForm_MouseUp(object? sender, MouseEventArgs e)
        {
            if (_config.ClickThroughMode) return;

            _dragging = false;
        }

        protected override void OnPaint(PaintEventArgs e)
        {
            base.OnPaint(e);

            // Ensure the label gets painted with outline
            _label.Invalidate();
        }
    }
}