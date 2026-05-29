use windows::core::{BSTR, GUID, PCWSTR};
use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::{
    eCapture, eCommunications, IMMDevice, IMMDeviceEnumerator, MMDeviceEnumerator,
    DEVICE_STATE_ACTIVE,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoTaskMemFree, CLSCTX_ALL, STGM_READ,
};

use crate::util::from_pwstr;

#[derive(Debug, Clone)]
pub struct MicInfo {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

impl MicInfo {
    pub fn display_name(&self) -> String {
        if self.is_default {
            format!("{} (Default)", self.name)
        } else {
            self.name.clone()
        }
    }
}

/// Controls mute state of a selected (or default) capture endpoint via WASAPI.
pub struct MicController {
    enumerator: IMMDeviceEnumerator,
    volume: Option<IAudioEndpointVolume>,
}

impl MicController {
    /// Create the controller. Returns `None` only if the device enumerator
    /// itself cannot be created (COM not initialized / unavailable).
    pub fn new(selected_id: &str) -> Option<MicController> {
        unsafe {
            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()?;
            let mut ctrl = MicController {
                enumerator,
                volume: None,
            };
            ctrl.set_microphone(selected_id);
            Some(ctrl)
        }
    }

    /// Select a capture device by id, falling back to the default
    /// communications device when the id is empty or invalid.
    pub fn set_microphone(&mut self, selected_id: &str) {
        unsafe {
            self.volume = None;
            let device = self.resolve_device(selected_id);
            if let Some(device) = device {
                if let Ok(vol) = device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None) {
                    self.volume = Some(vol);
                }
            }
        }
    }

    unsafe fn resolve_device(&self, selected_id: &str) -> Option<IMMDevice> {
        if !selected_id.is_empty() {
            let id_wide: Vec<u16> = selected_id
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            if let Ok(dev) = self.enumerator.GetDevice(PCWSTR(id_wide.as_ptr())) {
                return Some(dev);
            }
        }
        self.enumerator
            .GetDefaultAudioEndpoint(eCapture, eCommunications)
            .ok()
    }

    pub fn is_muted(&self) -> bool {
        match &self.volume {
            Some(vol) => unsafe { vol.GetMute().map(|b| b.as_bool()).unwrap_or(false) },
            None => false,
        }
    }

    pub fn set_mute(&self, mute: bool) {
        if let Some(vol) = &self.volume {
            unsafe {
                let _ = vol.SetMute(mute, &GUID::zeroed());
            }
        }
    }

    /// Toggle mute and return the resulting state.
    pub fn toggle_mute(&self) -> bool {
        let new_state = !self.is_muted();
        self.set_mute(new_state);
        new_state
    }

    pub fn available_microphones(&self) -> Vec<MicInfo> {
        let mut mics = Vec::new();
        unsafe {
            let default_id = self
                .enumerator
                .GetDefaultAudioEndpoint(eCapture, eCommunications)
                .ok()
                .and_then(|d| device_id(&d));

            if let Ok(collection) = self
                .enumerator
                .EnumAudioEndpoints(eCapture, DEVICE_STATE_ACTIVE)
            {
                if let Ok(count) = collection.GetCount() {
                    for i in 0..count {
                        if let Ok(device) = collection.Item(i) {
                            if let Some(id) = device_id(&device) {
                                let name = device_friendly_name(&device)
                                    .unwrap_or_else(|| "Unknown device".to_string());
                                let is_default = default_id.as_deref() == Some(id.as_str());
                                mics.push(MicInfo {
                                    id,
                                    name,
                                    is_default,
                                });
                            }
                        }
                    }
                }
            }
        }
        // Default device first, then alphabetical.
        mics.sort_by(|a, b| {
            b.is_default
                .cmp(&a.is_default)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
        mics
    }
}

unsafe fn device_id(device: &IMMDevice) -> Option<String> {
    match device.GetId() {
        Ok(pwstr) => {
            let s = from_pwstr(pwstr);
            CoTaskMemFree(Some(pwstr.0 as *const _));
            Some(s)
        }
        Err(_) => None,
    }
}

unsafe fn device_friendly_name(device: &IMMDevice) -> Option<String> {
    let store = device.OpenPropertyStore(STGM_READ).ok()?;
    let prop = store.GetValue(&PKEY_Device_FriendlyName).ok()?;
    // The friendly name is stored as VT_LPWSTR; convert through BSTR. The
    // PROPVARIANT is cleared automatically when it drops.
    let bstr = BSTR::try_from(&prop).ok()?;
    let s = bstr.to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}
