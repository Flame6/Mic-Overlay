# MicMuteOverlay

A simple Windows Forms utility that displays microphone mute status and lets you toggle it with a global hotkey.

## Building

Requires .NET 6 SDK. Open the solution `MicMuteOverlay.sln` in Visual Studio 2022 or run:

```bash
cd MicMuteOverlay
dotnet build
```

## Usage

Run `MicMuteOverlay.exe`. By default press **Ctrl+Shift+M** to toggle mute. An always-on-top transparent overlay will show when the microphone is muted. Right-click the overlay to open the settings window. Settings are stored in `config.json`.

You can customise the hotkey, overlay text, sound file paths and text appearance in the settings window. Sound files are located in the `Sounds` folder.
