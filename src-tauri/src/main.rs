//! TTRPG Companion - Real-time audio companion app for tabletop RPG sessions
//!
//! This application provides voice input processing for TTRPG game masters,
//! including speech-to-text transcription and emotional tone analysis.

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use ttrpg_companion_lib::run;

fn main() {
    run();
}
