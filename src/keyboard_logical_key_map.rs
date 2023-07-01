use winit::keyboard::Key;

pub fn translate_logical_key(key: &Key) -> Option<u64> {
    Some(match key {
        Key::Character(ch) => {
            let mut iter = ch.chars();
            match ch.len() {
                0 => return None,
                1 => iter.next().unwrap() as u64,
                2 => (iter.next().unwrap() as u64) | (iter.next().unwrap() as u64) << 8,
                3 => {
                    (iter.next().unwrap() as u64)
                        | (iter.next().unwrap() as u64) << 8
                        | (iter.next().unwrap() as u64) << 16
                }
                4 => {
                    (iter.next().unwrap() as u64)
                        | (iter.next().unwrap() as u64) << 8
                        | (iter.next().unwrap() as u64) << 16
                        | (iter.next().unwrap() as u64) << 24
                }
                _ => return None,
            }
        }
        Key::Space => 0x00000000020,
        Key::Backspace => 0x00100000008,
        Key::Tab => 0x00100000009,
        Key::Enter => 0x0010000000d,
        Key::Escape => 0x0010000001b,
        Key::Delete => 0x0010000007f,
        // Key::Accel => 0x00100000101,
        Key::AltGraph => 0x00100000103,
        Key::CapsLock => 0x00100000104,
        Key::Fn => 0x00100000106,
        Key::FnLock => 0x00100000107,
        Key::Hyper => 0x00100000108,
        Key::NumLock => 0x0010000010a,
        Key::ScrollLock => 0x0010000010c,
        Key::Super => 0x0010000010e,
        Key::Symbol => 0x0010000010f,
        Key::SymbolLock => 0x00100000110,
        // Key::ShiftLevel5 => 0x00100000111,
        Key::ArrowDown => 0x00100000301,
        Key::ArrowLeft => 0x00100000302,
        Key::ArrowRight => 0x00100000303,
        Key::ArrowUp => 0x00100000304,
        Key::End => 0x00100000305,
        Key::Home => 0x00100000306,
        Key::PageDown => 0x00100000307,
        Key::PageUp => 0x00100000308,
        Key::Clear => 0x00100000401,
        Key::Copy => 0x00100000402,
        Key::CrSel => 0x00100000403,
        Key::Cut => 0x00100000404,
        Key::EraseEof => 0x00100000405,
        Key::ExSel => 0x00100000406,
        Key::Insert => 0x00100000407,
        Key::Paste => 0x00100000408,
        Key::Redo => 0x00100000409,
        Key::Undo => 0x0010000040a,
        Key::Accept => 0x00100000501,
        Key::Again => 0x00100000502,
        Key::Attn => 0x00100000503,
        Key::Cancel => 0x00100000504,
        Key::ContextMenu => 0x00100000505,
        Key::Execute => 0x00100000506,
        Key::Find => 0x00100000507,
        Key::Help => 0x00100000508,
        Key::Pause => 0x00100000509,
        Key::Play => 0x0010000050a,
        Key::Props => 0x0010000050b,
        Key::Select => 0x0010000050c,
        Key::ZoomIn => 0x0010000050d,
        Key::ZoomOut => 0x0010000050e,
        Key::BrightnessDown => 0x00100000601,
        Key::BrightnessUp => 0x00100000602,
        Key::Camera => 0x00100000603,
        Key::Eject => 0x00100000604,
        Key::LogOff => 0x00100000605,
        Key::Power => 0x00100000606,
        Key::PowerOff => 0x00100000607,
        Key::PrintScreen => 0x00100000608,
        Key::Hibernate => 0x00100000609,
        Key::Standby => 0x0010000060a,
        Key::WakeUp => 0x0010000060b,
        Key::AllCandidates => 0x00100000701,
        Key::Alphanumeric => 0x00100000702,
        Key::CodeInput => 0x00100000703,
        Key::Compose => 0x00100000704,
        Key::Convert => 0x00100000705,
        Key::FinalMode => 0x00100000706,
        Key::GroupFirst => 0x00100000707,
        Key::GroupLast => 0x00100000708,
        Key::GroupNext => 0x00100000709,
        Key::GroupPrevious => 0x0010000070a,
        Key::ModeChange => 0x0010000070b,
        Key::NextCandidate => 0x0010000070c,
        Key::NonConvert => 0x0010000070d,
        Key::PreviousCandidate => 0x0010000070e,
        Key::Process => 0x0010000070f,
        Key::SingleCandidate => 0x00100000710,
        Key::HangulMode => 0x00100000711,
        Key::HanjaMode => 0x00100000712,
        Key::JunjaMode => 0x00100000713,
        Key::Eisu => 0x00100000714,
        Key::Hankaku => 0x00100000715,
        Key::Hiragana => 0x00100000716,
        Key::HiraganaKatakana => 0x00100000717,
        Key::KanaMode => 0x00100000718,
        Key::KanjiMode => 0x00100000719,
        Key::Katakana => 0x0010000071a,
        Key::Romaji => 0x0010000071b,
        Key::Zenkaku => 0x0010000071c,
        Key::ZenkakuHankaku => 0x0010000071d,
        Key::F1 => 0x00100000801,
        Key::F2 => 0x00100000802,
        Key::F3 => 0x00100000803,
        Key::F4 => 0x00100000804,
        Key::F5 => 0x00100000805,
        Key::F6 => 0x00100000806,
        Key::F7 => 0x00100000807,
        Key::F8 => 0x00100000808,
        Key::F9 => 0x00100000809,
        Key::F10 => 0x0010000080a,
        Key::F11 => 0x0010000080b,
        Key::F12 => 0x0010000080c,
        Key::F13 => 0x0010000080d,
        Key::F14 => 0x0010000080e,
        Key::F15 => 0x0010000080f,
        Key::F16 => 0x00100000810,
        Key::F17 => 0x00100000811,
        Key::F18 => 0x00100000812,
        Key::F19 => 0x00100000813,
        Key::F20 => 0x00100000814,
        Key::F21 => 0x00100000815,
        Key::F22 => 0x00100000816,
        Key::F23 => 0x00100000817,
        Key::F24 => 0x00100000818,
        Key::Soft1 => 0x00100000901,
        Key::Soft2 => 0x00100000902,
        Key::Soft3 => 0x00100000903,
        Key::Soft4 => 0x00100000904,
        // Key::Soft5 => 0x00100000905,
        // Key::Soft6 => 0x00100000906,
        // Key::Soft7 => 0x00100000907,
        // Key::Soft8 => 0x00100000908,
        Key::Close => 0x00100000a01,
        Key::MailForward => 0x00100000a02,
        Key::MailReply => 0x00100000a03,
        Key::MailSend => 0x00100000a04,
        Key::MediaPlayPause => 0x00100000a05,
        Key::MediaStop => 0x00100000a07,
        Key::MediaTrackNext => 0x00100000a08,
        Key::MediaTrackPrevious => 0x00100000a09,
        Key::New => 0x00100000a0a,
        Key::Open => 0x00100000a0b,
        Key::Print => 0x00100000a0c,
        Key::Save => 0x00100000a0d,
        Key::SpellCheck => 0x00100000a0e,
        Key::AudioVolumeDown => 0x00100000a0f,
        Key::AudioVolumeUp => 0x00100000a10,
        Key::AudioVolumeMute => 0x00100000a11,
        Key::LaunchApplication2 => 0x00100000b01,
        Key::LaunchCalendar => 0x00100000b02,
        Key::LaunchMail => 0x00100000b03,
        Key::LaunchMediaPlayer => 0x00100000b04,
        Key::LaunchMusicPlayer => 0x00100000b05,
        Key::LaunchApplication1 => 0x00100000b06,
        Key::LaunchScreenSaver => 0x00100000b07,
        Key::LaunchSpreadsheet => 0x00100000b08,
        Key::LaunchWebBrowser => 0x00100000b09,
        Key::LaunchWebCam => 0x00100000b0a,
        Key::LaunchWordProcessor => 0x00100000b0b,
        Key::LaunchContacts => 0x00100000b0c,
        Key::LaunchPhone => 0x00100000b0d,
        // Key::LaunchAssistant => 0x00100000b0e,
        // Key::LaunchControlPanel => 0x00100000b0f,
        Key::BrowserBack => 0x00100000c01,
        Key::BrowserFavorites => 0x00100000c02,
        Key::BrowserForward => 0x00100000c03,
        Key::BrowserHome => 0x00100000c04,
        Key::BrowserRefresh => 0x00100000c05,
        Key::BrowserSearch => 0x00100000c06,
        Key::BrowserStop => 0x00100000c07,
        Key::AudioBalanceLeft => 0x00100000d01,
        Key::AudioBalanceRight => 0x00100000d02,
        Key::AudioBassBoostDown => 0x00100000d03,
        Key::AudioBassBoostUp => 0x00100000d04,
        Key::AudioFaderFront => 0x00100000d05,
        Key::AudioFaderRear => 0x00100000d06,
        Key::AudioSurroundModeNext => 0x00100000d07,
        Key::AVRInput => 0x00100000d08,
        Key::AVRPower => 0x00100000d09,
        Key::ChannelDown => 0x00100000d0a,
        Key::ChannelUp => 0x00100000d0b,
        Key::ColorF0Red => 0x00100000d0c,
        Key::ColorF1Green => 0x00100000d0d,
        Key::ColorF2Yellow => 0x00100000d0e,
        Key::ColorF3Blue => 0x00100000d0f,
        Key::ColorF4Grey => 0x00100000d10,
        Key::ColorF5Brown => 0x00100000d11,
        Key::ClosedCaptionToggle => 0x00100000d12,
        Key::Dimmer => 0x00100000d13,
        Key::DisplaySwap => 0x00100000d14,
        Key::Exit => 0x00100000d15,
        Key::FavoriteClear0 => 0x00100000d16,
        Key::FavoriteClear1 => 0x00100000d17,
        Key::FavoriteClear2 => 0x00100000d18,
        Key::FavoriteClear3 => 0x00100000d19,
        Key::FavoriteRecall0 => 0x00100000d1a,
        Key::FavoriteRecall1 => 0x00100000d1b,
        Key::FavoriteRecall2 => 0x00100000d1c,
        Key::FavoriteRecall3 => 0x00100000d1d,
        Key::FavoriteStore0 => 0x00100000d1e,
        Key::FavoriteStore1 => 0x00100000d1f,
        Key::FavoriteStore2 => 0x00100000d20,
        Key::FavoriteStore3 => 0x00100000d21,
        Key::Guide => 0x00100000d22,
        Key::GuideNextDay => 0x00100000d23,
        Key::GuidePreviousDay => 0x00100000d24,
        Key::Info => 0x00100000d25,
        Key::InstantReplay => 0x00100000d26,
        Key::Link => 0x00100000d27,
        Key::ListProgram => 0x00100000d28,
        Key::LiveContent => 0x00100000d29,
        Key::Lock => 0x00100000d2a,
        Key::MediaApps => 0x00100000d2b,
        Key::MediaFastForward => 0x00100000d2c,
        Key::MediaLast => 0x00100000d2d,
        Key::MediaPause => 0x00100000d2e,
        Key::MediaPlay => 0x00100000d2f,
        Key::MediaRecord => 0x00100000d30,
        Key::MediaRewind => 0x00100000d31,
        Key::NextFavoriteChannel => 0x00100000d33,
        Key::NextUserProfile => 0x00100000d34,
        Key::OnDemand => 0x00100000d35,
        Key::PinPDown => 0x00100000d36,
        Key::PinPMove => 0x00100000d37,
        Key::PinPToggle => 0x00100000d38,
        Key::PinPUp => 0x00100000d39,
        Key::PlaySpeedDown => 0x00100000d3a,
        Key::PlaySpeedReset => 0x00100000d3b,
        Key::PlaySpeedUp => 0x00100000d3c,
        Key::RandomToggle => 0x00100000d3d,
        Key::RcLowBattery => 0x00100000d3e,
        Key::RecordSpeedNext => 0x00100000d3f,
        Key::RfBypass => 0x00100000d40,
        Key::ScanChannelsToggle => 0x00100000d41,
        Key::ScreenModeNext => 0x00100000d42,
        Key::Settings => 0x00100000d43,
        Key::SplitScreenToggle => 0x00100000d44,
        Key::STBInput => 0x00100000d45,
        Key::STBPower => 0x00100000d46,
        Key::Subtitle => 0x00100000d47,
        Key::Teletext => 0x00100000d48,
        Key::TV => 0x00100000d49,
        Key::TVInput => 0x00100000d4a,
        Key::TVPower => 0x00100000d4b,
        Key::VideoModeNext => 0x00100000d4c,
        Key::Wink => 0x00100000d4d,
        Key::ZoomToggle => 0x00100000d4e,
        Key::DVR => 0x00100000d4f,
        Key::MediaAudioTrack => 0x00100000d50,
        Key::MediaSkipBackward => 0x00100000d51,
        Key::MediaSkipForward => 0x00100000d52,
        Key::MediaStepBackward => 0x00100000d53,
        Key::MediaStepForward => 0x00100000d54,
        Key::MediaTopMenu => 0x00100000d55,
        Key::NavigateIn => 0x00100000d56,
        Key::NavigateNext => 0x00100000d57,
        Key::NavigateOut => 0x00100000d58,
        Key::NavigatePrevious => 0x00100000d59,
        Key::Pairing => 0x00100000d5a,
        Key::MediaClose => 0x00100000d5b,
        Key::AudioBassBoostToggle => 0x00100000e02,
        Key::AudioTrebleDown => 0x00100000e04,
        Key::AudioTrebleUp => 0x00100000e05,
        Key::MicrophoneToggle => 0x00100000e06,
        Key::MicrophoneVolumeDown => 0x00100000e07,
        Key::MicrophoneVolumeUp => 0x00100000e08,
        Key::MicrophoneVolumeMute => 0x00100000e09,
        Key::SpeechCorrectionList => 0x00100000f01,
        Key::SpeechInputToggle => 0x00100000f02,
        Key::AppSwitch => 0x00100001001,
        Key::Call => 0x00100001002,
        Key::CameraFocus => 0x00100001003,
        Key::EndCall => 0x00100001004,
        Key::GoBack => 0x00100001005,
        Key::GoHome => 0x00100001006,
        Key::HeadsetHook => 0x00100001007,
        Key::LastNumberRedial => 0x00100001008,
        Key::Notification => 0x00100001009,
        Key::MannerMode => 0x0010000100a,
        Key::VoiceDial => 0x0010000100b,
        Key::TV3DMode => 0x00100001101,
        Key::TVAntennaCable => 0x00100001102,
        Key::TVAudioDescription => 0x00100001103,
        Key::TVAudioDescriptionMixDown => 0x00100001104,
        Key::TVAudioDescriptionMixUp => 0x00100001105,
        Key::TVContentsMenu => 0x00100001106,
        Key::TVDataService => 0x00100001107,
        Key::TVInputComponent1 => 0x00100001108,
        Key::TVInputComponent2 => 0x00100001109,
        Key::TVInputComposite1 => 0x0010000110a,
        Key::TVInputComposite2 => 0x0010000110b,
        Key::TVInputHDMI1 => 0x0010000110c,
        Key::TVInputHDMI2 => 0x0010000110d,
        Key::TVInputHDMI3 => 0x0010000110e,
        Key::TVInputHDMI4 => 0x0010000110f,
        Key::TVInputVGA1 => 0x00100001110,
        Key::TVMediaContext => 0x00100001111,
        Key::TVNetwork => 0x00100001112,
        Key::TVNumberEntry => 0x00100001113,
        Key::TVRadioService => 0x00100001114,
        Key::TVSatellite => 0x00100001115,
        Key::TVSatelliteBS => 0x00100001116,
        Key::TVSatelliteCS => 0x00100001117,
        Key::TVSatelliteToggle => 0x00100001118,
        Key::TVTerrestrialAnalog => 0x00100001119,
        Key::TVTerrestrialDigital => 0x0010000111a,
        Key::TVTimer => 0x0010000111b,
        Key::Key11 => 0x00100001201,
        Key::Key12 => 0x00100001202,
        // Key::Standby => 0x00200000000,
        // Key::WakeUp => 0x00200000001,
        // Key::Sleep => 0x00200000002,
        // Key::Abort => 0x00200000003,
        // Key::Lang1 => 0x00200000010,
        // Key::Lang2 => 0x00200000011,
        // Key::Lang3 => 0x00200000012,
        // Key::Lang4 => 0x00200000013,
        // Key::Lang5 => 0x00200000014,
        Key::Control => 0x00200000100,
        Key::Shift => 0x00200000102,
        Key::Alt => 0x00200000104,
        Key::Meta => 0x00200000106,
        // Key::NumpadEnter => 0x0020000020d,
        // Key::NumpadParenLeft => 0x00200000228,
        // Key::NumpadParenRight => 0x00200000229,
        // Key::NumpadMultiply => 0x0020000022a,
        // Key::NumpadAdd => 0x0020000022b,
        // Key::NumpadComma => 0x0020000022c,
        // Key::NumpadSubtract => 0x0020000022d,
        // Key::NumpadDecimal => 0x0020000022e,
        // Key::NumpadDivide => 0x0020000022f,
        // Key::Numpad0 => 0x00200000230,
        // Key::Numpad1 => 0x00200000231,
        // Key::Numpad2 => 0x00200000232,
        // Key::Numpad3 => 0x00200000233,
        // Key::Numpad4 => 0x00200000234,
        // Key::Numpad5 => 0x00200000235,
        // Key::Numpad6 => 0x00200000236,
        // Key::Numpad7 => 0x00200000237,
        // Key::Numpad8 => 0x00200000238,
        // Key::Numpad9 => 0x00200000239,
        // Key::NumpadEqual => 0x0020000023d,
        // Key::GameButton1 => 0x00200000301,
        // Key::GameButton2 => 0x00200000302,
        // Key::GameButton3 => 0x00200000303,
        // Key::GameButton4 => 0x00200000304,
        // Key::GameButton5 => 0x00200000305,
        // Key::GameButton6 => 0x00200000306,
        // Key::GameButton7 => 0x00200000307,
        // Key::GameButton8 => 0x00200000308,
        // Key::GameButton9 => 0x00200000309,
        // Key::GameButton10 => 0x0020000030a,
        // Key::GameButton11 => 0x0020000030b,
        // Key::GameButton12 => 0x0020000030c,
        // Key::GameButton13 => 0x0020000030d,
        // Key::GameButton14 => 0x0020000030e,
        // Key::GameButton15 => 0x0020000030f,
        // Key::GameButton16 => 0x00200000310,
        // Key::GameButtonA => 0x00200000311,
        // Key::GameButtonB => 0x00200000312,
        // Key::GameButtonC => 0x00200000313,
        // Key::GameButtonLeft1 => 0x00200000314,
        // Key::GameButtonLeft2 => 0x00200000315,
        // Key::GameButtonMode => 0x00200000316,
        // Key::GameButtonRight1 => 0x00200000317,
        // Key::GameButtonRight2 => 0x00200000318,
        // Key::GameButtonSelect => 0x00200000319,
        // Key::GameButtonStart => 0x0020000031a,
        // Key::GameButtonThumbLeft => 0x0020000031b,
        // Key::GameButtonThumbRight => 0x0020000031c,
        // Key::GameButtonX => 0x0020000031d,
        // Key::GameButtonY => 0x0020000031e,
        // Key::GameButtonZ => 0x0020000031f,
        // Key::F25 => return None,
        // Key::F26 => return None,
        // Key::F27 => return None,
        // Key::F28 => return None,
        // Key::F29 => return None,
        // Key::F30 => return None,
        // Key::F31 => return None,
        // Key::F32 => return None,
        // Key::F33 => return None,
        // Key::F34 => return None,
        // Key::F35 => return None,
        // Key::Dead(_) => return None,
        _ => return None,
    })
}
