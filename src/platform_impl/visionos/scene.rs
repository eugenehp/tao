// Copyright 2023-2024 Eugene Hauptmann
// Copyright 2024-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0

use std::{collections::HashMap, ffi::c_char};

use objc::{
  declare::ClassDecl,
  runtime::{Class, Object, Sel, BOOL, NO, YES},
};

use crate::{
  dpi::PhysicalPosition,
  event::{DeviceId as RootDeviceId, Event, Force, Touch, TouchPhase, WindowEvent},
  platform::visionos::MonitorHandleExtVisionOS,
  platform_impl::platform::{
    app_state::{self, OSCapabilities},
    event_loop::{self, EventProxy, EventWrapper},
    ffi::{
      id, nil, CGFloat, CGPoint, CGRect, UIForceTouchCapability, UIInterfaceOrientationMask,
      UIRectEdge, UITouchPhase, UITouchType, NSStringRust
    },
    window::PlatformSpecificWindowBuilderAttributes,
    DeviceId,
  },
  window::{Fullscreen, WindowAttributes, WindowId as RootWindowId},
};

pub fn create_delegate_class() {
    
    extern "C" fn scene_will_connect_to_session_with_options(
        this: &Object,
        _: Sel,
        scene: id,
        session: id,
        options: id
    ) {
        println!("=======scene_will_connect_to_session_with_options");
        // TODO: write rust version of objc to create a scene and window
        // TODO(2): figure out how to use scene to load windows configured by Tauri

        // https://github.com/ryanmcgrath/cacao/blob/trunk/examples/ios-beta/main.rs#L119

        // guard let windowScene = scene as? UIWindowScene else {
		// 	fatalError("Expected scene of type UIWindowScene but got an unexpected type")
		// }
		
		// let size = CGSize(width: 480, height: 480)
		// windowScene.sizeRestrictions?.minimumSize = size
		// windowScene.sizeRestrictions?.maximumSize = size
		
		// window = UIWindow(windowScene: windowScene)		
		
		// if let window = window {
		// 	window.rootViewController = VVUMainViewController()

			
		// 	window.makeKeyAndVisible()
		// }
    }
    
    let ui_responder = class!(UIResponder);
    let mut decl = ClassDecl::new("TaoWindowSceneDelegate", ui_responder).expect("Failed to declare class `TaoWindowSceneDelegate`");

    unsafe {
        // UIWindowSceneDelegate API
        decl.add_method(
            sel!(scene:willConnectToSession:options:),
            scene_will_connect_to_session_with_options as extern "C" fn(&Object, Sel, id, id, id)
        );

        decl.register();
    }
}