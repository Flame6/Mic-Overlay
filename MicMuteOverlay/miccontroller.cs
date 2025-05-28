using System;
using NAudio.CoreAudioApi;

namespace MicMuteOverlay
{
    public class MicController : IDisposable
    {
        private readonly MMDevice _device;
        public bool IsMuted => _device.AudioEndpointVolume.Mute;

        public MicController()
        {
            var enumerator = new MMDeviceEnumerator();
            _device = enumerator.GetDefaultAudioEndpoint(DataFlow.Capture, Role.Communications);
        }

        public void ToggleMute()
        {
            _device.AudioEndpointVolume.Mute = !_device.AudioEndpointVolume.Mute;
        }

        public void Dispose()
        {
            _device?.Dispose();
        }
    }
}