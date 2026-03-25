// binge-watch-me — a self-hosted media remote controlled from your phone
// Copyright (C) 2026  Aleksandar Parvanov
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use coreaudio_sys::{
    AudioObjectGetPropertyData, AudioObjectSetPropertyData, AudioObjectPropertyAddress,
    AudioObjectPropertyScope, AudioObjectPropertyElement, AudioDeviceID,
    kAudioHardwarePropertyDefaultOutputDevice, kAudioObjectPropertyScopeGlobal,
    kAudioObjectPropertyElementMain, kAudioObjectSystemObject,
    kAudioHardwareServiceDeviceProperty_VirtualMainVolume,
    kAudioObjectPropertyScopeOutput,
};

/// Get the default output device ID from CoreAudio
fn get_default_output_device() -> Option<AudioDeviceID> {
    let property_address = AudioObjectPropertyAddress {
        mSelector: kAudioHardwarePropertyDefaultOutputDevice,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMain,
    };

    let mut device_id: AudioDeviceID = 0;
    let mut size = std::mem::size_of::<AudioDeviceID>() as u32;

    let result = unsafe {
        AudioObjectGetPropertyData(
            kAudioObjectSystemObject,
            &property_address,
            0,
            std::ptr::null(),
            &mut size,
            &mut device_id as *mut _ as *mut _,
        )
    };

    if result == 0 { Some(device_id) } else { None }
}

/// Get current volume as a float 0.0 - 1.0
fn get_volume() -> Option<f32> {
    let device_id = get_default_output_device()?;

    let property_address = AudioObjectPropertyAddress {
        mSelector: kAudioHardwareServiceDeviceProperty_VirtualMainVolume,
        mScope: kAudioObjectPropertyScopeOutput,
        mElement: kAudioObjectPropertyElementMain,
    };

    let mut volume: f32 = 0.0;
    let mut size = std::mem::size_of::<f32>() as u32;

    let result = unsafe {
        AudioObjectGetPropertyData(
            device_id,
            &property_address,
            0,
            std::ptr::null(),
            &mut size,
            &mut volume as *mut _ as *mut _,
        )
    };

    if result == 0 { Some(volume) } else { None }
}

/// Set volume as a float 0.0 - 1.0
fn set_volume_raw(volume: f32) {
    let Some(device_id) = get_default_output_device() else { return };

    let property_address = AudioObjectPropertyAddress {
        mSelector: kAudioHardwareServiceDeviceProperty_VirtualMainVolume,
        mScope: kAudioObjectPropertyScopeOutput,
        mElement: kAudioObjectPropertyElementMain,
    };

    let mut volume = volume.clamp(0.0, 1.0);
    let size = std::mem::size_of::<f32>() as u32;

    unsafe {
        AudioObjectSetPropertyData(
            device_id,
            &property_address,
            0,
            std::ptr::null(),
            size,
            &mut volume as *mut _ as *mut _,
        );
    }
}

// Public API — same interface as before, nothing else in the app changes

pub fn volume_up() {
    if let Some(current) = get_volume() {
        set_volume_raw((current + 0.05).min(1.0));
    }
}

pub fn volume_down() {
    if let Some(current) = get_volume() {
        set_volume_raw((current - 0.05).max(0.0));
    }
}

pub fn set_volume(level: u8) {
    set_volume_raw(level as f32 / 100.0);
}
