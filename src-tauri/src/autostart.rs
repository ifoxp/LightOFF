//! «Запускати з Windows»: a value under HKCU\…\Run that starts the app in the tray.

#[cfg(windows)]
mod imp {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE};
    use winreg::RegKey;

    const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    const NAME: &str = "LightOFF";

    pub fn enabled() -> bool {
        RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey_with_flags(RUN, KEY_READ)
            .and_then(|k| k.get_value::<String, _>(NAME))
            .is_ok()
    }

    pub fn set(on: bool) -> anyhow::Result<()> {
        let key = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(RUN, KEY_SET_VALUE)?;
        if on {
            let exe = std::env::current_exe()?;
            key.set_value(NAME, &format!("\"{}\" --tray", exe.display()))?;
        } else if enabled() {
            key.delete_value(NAME)?;
        }
        Ok(())
    }
}

#[cfg(not(windows))]
mod imp {
    pub fn enabled() -> bool {
        false
    }
    pub fn set(_on: bool) -> anyhow::Result<()> {
        anyhow::bail!("лише для Windows")
    }
}

pub use imp::{enabled, set};
