// Copyright 2014-2021 The winit contributors
// Copyright 2021-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
#![allow(non_snake_case)]

use once_cell::sync::Lazy;
/// This is a simple implementation of support for Windows Dark Mode,
/// which is inspired by the solution in https://github.com/ysc3839/win32-darkmode
use windows::{
  core::{s, w, PCSTR, PSTR},
  Win32::{
    Foundation::{BOOL, HANDLE, HMODULE, HWND},
    System::{LibraryLoader::*, SystemInformation::OSVERSIONINFOW},
    UI::{Accessibility::*, WindowsAndMessaging::*},
  },
};

use std::ffi::c_void;

use crate::window::Theme;

static HUXTHEME: Lazy<HMODULE> =
  Lazy::new(|| unsafe { LoadLibraryA(s!("uxtheme.dll")).unwrap_or_default() });

static WIN10_BUILD_VERSION: Lazy<Option<u32>> = Lazy::new(|| {
  type RtlGetVersion = unsafe extern "system" fn(*mut OSVERSIONINFOW) -> i32;

  let handle = get_function!("ntdll.dll", RtlGetVersion);

  let mut vi = OSVERSIONINFOW {
    dwOSVersionInfoSize: 0,
    dwMajorVersion: 0,
    dwMinorVersion: 0,
    dwBuildNumber: 0,
    dwPlatformId: 0,
    szCSDVersion: [0; 128],
  };

  if let Some(rtl_get_version) = handle {
    let status = unsafe { (rtl_get_version)(&mut vi as _) };

    if status >= 0 && vi.dwMajorVersion == 10 && vi.dwMinorVersion == 0 {
      Some(vi.dwBuildNumber)
    } else {
      None
    }
  } else {
    None
  }
});

static DARK_MODE_SUPPORTED: Lazy<bool> = Lazy::new(|| {
  // We won't try to do anything for windows versions < 17763
  // (Windows 10 October 2018 update)
  match *WIN10_BUILD_VERSION {
    Some(v) => v >= 17763,
    None => false,
  }
});

/// Attempts to set dark mode for the app
pub fn try_app_theme(preferred_theme: Option<Theme>) -> Theme {
  if *DARK_MODE_SUPPORTED {
    let is_dark_mode = match preferred_theme {
      Some(theme) => theme == Theme::Dark,
      None => should_use_dark_mode(),
    };

    allow_dark_mode_for_app(is_dark_mode);
    refresh_immersive_color_policy_state();
    match is_dark_mode {
      true => Theme::Dark,
      false => Theme::Light,
    }
  } else {
    Theme::Light
  }
}

fn allow_dark_mode_for_app(is_dark_mode: bool) {
  const UXTHEME_ALLOWDARKMODEFORAPP_ORDINAL: u16 = 135;
  type AllowDarkModeForApp = unsafe extern "system" fn(bool) -> bool;
  static ALLOW_DARK_MODE_FOR_APP: Lazy<Option<AllowDarkModeForApp>> = Lazy::new(|| unsafe {
    if HUXTHEME.is_invalid() {
      return None;
    }

    GetProcAddress(
      *HUXTHEME,
      PCSTR::from_raw(UXTHEME_ALLOWDARKMODEFORAPP_ORDINAL as usize as *mut _),
    )
    .map(|handle| std::mem::transmute(handle))
  });

  #[repr(C)]
  enum PreferredAppMode {
    Default,
    AllowDark,
    // ForceDark,
    // ForceLight,
    // Max,
  }
  const UXTHEME_SETPREFERREDAPPMODE_ORDINAL: u16 = 135;
  type SetPreferredAppMode = unsafe extern "system" fn(PreferredAppMode) -> PreferredAppMode;
  static SET_PREFERRED_APP_MODE: Lazy<Option<SetPreferredAppMode>> = Lazy::new(|| unsafe {
    if HUXTHEME.is_invalid() {
      return None;
    }

    GetProcAddress(
      *HUXTHEME,
      PCSTR::from_raw(UXTHEME_SETPREFERREDAPPMODE_ORDINAL as usize as *mut _),
    )
    .map(|handle| std::mem::transmute(handle))
  });

  if let Some(ver) = *WIN10_BUILD_VERSION {
    if ver < 18362 {
      if let Some(_allow_dark_mode_for_app) = *ALLOW_DARK_MODE_FOR_APP {
        unsafe { _allow_dark_mode_for_app(is_dark_mode) };
      }
    } else if let Some(_set_preferred_app_mode) = *SET_PREFERRED_APP_MODE {
      let mode = if is_dark_mode {
        PreferredAppMode::AllowDark
      } else {
        PreferredAppMode::Default
      };
      unsafe { _set_preferred_app_mode(mode) };
    }
  }
}

fn refresh_immersive_color_policy_state() {
  const UXTHEME_REFRESHIMMERSIVECOLORPOLICYSTATE_ORDINAL: u16 = 104;
  type RefreshImmersiveColorPolicyState = unsafe extern "system" fn();
  static REFRESH_IMMERSIVE_COLOR_POLICY_STATE: Lazy<Option<RefreshImmersiveColorPolicyState>> =
    Lazy::new(|| unsafe {
      if HUXTHEME.is_invalid() {
        return None;
      }

      GetProcAddress(
        *HUXTHEME,
        PCSTR::from_raw(UXTHEME_REFRESHIMMERSIVECOLORPOLICYSTATE_ORDINAL as usize as *mut _),
      )
      .map(|handle| std::mem::transmute(handle))
    });

  if let Some(_refresh_immersive_color_policy_state) = *REFRESH_IMMERSIVE_COLOR_POLICY_STATE {
    unsafe { _refresh_immersive_color_policy_state() }
  }
}

/// Attempt to set a theme on a window, if necessary.
/// Returns the theme that was picked
pub fn try_window_theme(hwnd: HWND, preferred_theme: Option<Theme>) -> Theme {
  if *DARK_MODE_SUPPORTED {
    let is_dark_mode = match preferred_theme {
      Some(theme) => theme == Theme::Dark,
      None => should_use_dark_mode(),
    };

    let theme = if is_dark_mode {
      Theme::Dark
    } else {
      Theme::Light
    };

    allow_dark_mode_for_window(hwnd, is_dark_mode);
    refresh_titlebar_theme_color(hwnd);

    theme
  } else {
    Theme::Light
  }
}

fn allow_dark_mode_for_window(hwnd: HWND, is_dark_mode: bool) {
  const UXTHEME_ALLOWDARKMODEFORWINDOW_ORDINAL: u16 = 133;
  type AllowDarkModeForWindow = unsafe extern "system" fn(HWND, bool) -> bool;
  static ALLOW_DARK_MODE_FOR_WINDOW: Lazy<Option<AllowDarkModeForWindow>> = Lazy::new(|| unsafe {
    if HUXTHEME.is_invalid() {
      return None;
    }

    GetProcAddress(
      *HUXTHEME,
      PCSTR::from_raw(UXTHEME_ALLOWDARKMODEFORWINDOW_ORDINAL as usize as *mut _),
    )
    .map(|handle| std::mem::transmute(handle))
  });

  if *DARK_MODE_SUPPORTED {
    if let Some(_allow_dark_mode_for_window) = *ALLOW_DARK_MODE_FOR_WINDOW {
      unsafe { _allow_dark_mode_for_window(hwnd, is_dark_mode) };
    }
  }
}

fn is_dark_mode_allowed_for_window(hwnd: HWND) -> bool {
  const UXTHEME_ISDARKMODEALLOWEDFORWINDOW_ORDINAL: u16 = 137;
  type IsDarkModeAllowedForWindow = unsafe extern "system" fn(HWND) -> bool;
  static IS_DARK_MODE_ALLOWED_FOR_WINDOW: Lazy<Option<IsDarkModeAllowedForWindow>> =
    Lazy::new(|| unsafe {
      if HUXTHEME.is_invalid() {
        return None;
      }

      GetProcAddress(
        *HUXTHEME,
        PCSTR::from_raw(UXTHEME_ISDARKMODEALLOWEDFORWINDOW_ORDINAL as usize as *mut _),
      )
      .map(|handle| std::mem::transmute(handle))
    });

  if let Some(_is_dark_mode_allowed_for_window) = *IS_DARK_MODE_ALLOWED_FOR_WINDOW {
    unsafe { _is_dark_mode_allowed_for_window(hwnd) }
  } else {
    false
  }
}

type SetWindowCompositionAttribute =
  unsafe extern "system" fn(HWND, *mut WINDOWCOMPOSITIONATTRIBDATA) -> BOOL;
static SET_WINDOW_COMPOSITION_ATTRIBUTE: Lazy<Option<SetWindowCompositionAttribute>> =
  Lazy::new(|| get_function!("user32.dll", SetWindowCompositionAttribute));

type WINDOWCOMPOSITIONATTRIB = u32;
const WCA_USEDARKMODECOLORS: WINDOWCOMPOSITIONATTRIB = 26;
#[repr(C)]
struct WINDOWCOMPOSITIONATTRIBDATA {
  Attrib: WINDOWCOMPOSITIONATTRIB,
  pvData: *mut c_void,
  cbData: usize,
}

fn refresh_titlebar_theme_color(hwnd: HWND) {
  let dark = should_use_dark_mode() && is_dark_mode_allowed_for_window(hwnd);
  let mut is_dark_mode_bigbool: BOOL = dark.into();

  if let Some(ver) = *WIN10_BUILD_VERSION {
    if ver < 18362 {
      unsafe {
        let _ = SetPropW(
          hwnd,
          w!("UseImmersiveDarkModeColors"),
          HANDLE(&mut is_dark_mode_bigbool as *mut _ as _),
        );
      }
    } else if let Some(set_window_composition_attribute) = *SET_WINDOW_COMPOSITION_ATTRIBUTE {
      let mut data = WINDOWCOMPOSITIONATTRIBDATA {
        Attrib: WCA_USEDARKMODECOLORS,
        pvData: &mut is_dark_mode_bigbool as *mut _ as _,
        cbData: std::mem::size_of_val(&is_dark_mode_bigbool) as _,
      };
      unsafe { set_window_composition_attribute(hwnd, &mut data as *mut _) };
    }
  }
}

fn should_use_dark_mode() -> bool {
  should_apps_use_dark_mode() && !is_high_contrast()
}

fn should_apps_use_dark_mode() -> bool {
  const UXTHEME_SHOULDAPPSUSEDARKMODE_ORDINAL: u16 = 132;
  type ShouldAppsUseDarkMode = unsafe extern "system" fn() -> bool;
  static SHOULD_APPS_USE_DARK_MODE: Lazy<Option<ShouldAppsUseDarkMode>> = Lazy::new(|| unsafe {
    if HUXTHEME.is_invalid() {
      return None;
    }

    GetProcAddress(
      *HUXTHEME,
      PCSTR::from_raw(UXTHEME_SHOULDAPPSUSEDARKMODE_ORDINAL as usize as *mut _),
    )
    .map(|handle| std::mem::transmute(handle))
  });

  SHOULD_APPS_USE_DARK_MODE
    .map(|should_apps_use_dark_mode| unsafe { (should_apps_use_dark_mode)() })
    .unwrap_or(false)
}

fn is_high_contrast() -> bool {
  const HCF_HIGHCONTRASTON: u32 = 1;

  let mut hc = HIGHCONTRASTA {
    cbSize: 0,
    dwFlags: Default::default(),
    lpszDefaultScheme: PSTR::null(),
  };

  let ok = unsafe {
    SystemParametersInfoA(
      SPI_GETHIGHCONTRAST,
      std::mem::size_of_val(&hc) as _,
      Some(&mut hc as *mut _ as _),
      Default::default(),
    )
  };

  ok.is_ok() && (HCF_HIGHCONTRASTON & hc.dwFlags.0) != 0
}
