using System;
using System.Collections.Generic;
using System.Linq;
using NAudio.CoreAudioApi;

namespace MicMuteOverlay
{
    public class MicController : IDisposable
    {
        private MMDevice? _device;
        private readonly MMDeviceEnumerator _enumerator;

        public bool IsMuted => _device?.AudioEndpointVolume.Mute ?? false;

        public MicController(string? selectedDeviceId = null)
        {
            _enumerator = new MMDeviceEnumerator();
            SetMicrophone(selectedDeviceId);
        }

        public void SetMicrophone(string? deviceId)
        {
            _device?.Dispose();
            _device = null;

            try
            {
                if (!string.IsNullOrEmpty(deviceId))
                {
                    // Try to use the selected device
                    _device = _enumerator.GetDevice(deviceId);
                }
            }
            catch
            {
                // If selected device fails, fall back to default
                _device = null;
            }

            // If no device selected or selected device failed, use default
            if (_device == null)
            {
                try
                {
                    _device = _enumerator.GetDefaultAudioEndpoint(DataFlow.Capture, Role.Communications);
                }
                catch
                {
                    // No microphones available
                    _device = null;
                }
            }
        }

        public List<MicrophoneInfo> GetAvailableMicrophones()
        {
            var microphones = new List<MicrophoneInfo>();

            try
            {
                var devices = _enumerator.EnumerateAudioEndPoints(DataFlow.Capture, DeviceState.Active);

                foreach (var device in devices)
                {
                    microphones.Add(new MicrophoneInfo
                    {
                        Id = device.ID,
                        Name = device.FriendlyName,
                        IsDefault = IsDefaultDevice(device)
                    });
                }
            }
            catch
            {
                // Handle any errors getting device list
            }

            return microphones.OrderByDescending(m => m.IsDefault).ThenBy(m => m.Name).ToList();
        }

        private bool IsDefaultDevice(MMDevice device)
        {
            try
            {
                var defaultDevice = _enumerator.GetDefaultAudioEndpoint(DataFlow.Capture, Role.Communications);
                return device.ID == defaultDevice.ID;
            }
            catch
            {
                return false;
            }
        }

        public void ToggleMute()
        {
            if (_device != null)
            {
                _device.AudioEndpointVolume.Mute = !_device.AudioEndpointVolume.Mute;
            }
        }

        public void Dispose()
        {
            _device?.Dispose();
            _enumerator?.Dispose();
        }
    }

    public class MicrophoneInfo
    {
        public string Id { get; set; } = "";
        public string Name { get; set; } = "";
        public bool IsDefault { get; set; }

        public string DisplayName => IsDefault ? $"{Name} (Default)" : Name;

        public override string ToString()
        {
            return DisplayName;
        }
    }
}