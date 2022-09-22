#![allow(dead_code)]
#![feature(once_cell, result_option_inspect)]
use std::{path::PathBuf, sync::Arc};

use clap::Parser;
use tokio::runtime::Builder;
use wgpu::{
    Backends, DeviceDescriptor, Features, Instance, Limits, PowerPreference, PresentMode,
    RequestAdapterOptions, SurfaceConfiguration, TextureFormat, TextureUsages,
};
use winit::{
    dpi::PhysicalPosition,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
    window::{Window, WindowBuilder},
};

mod flutter_application;
mod test_platform_view;
use flutter_application::{FlutterApplication, FlutterApplicationCallback};

use crate::test_platform_view::TestPlatformView;

mod action_key;
mod keyboard_logical_key_map;
mod keyboard_physical_key_map;

mod flutter_bindings;
mod utils;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The Flutter application code needs to be snapshotted using
    /// the Flutter tools and the assets packaged in the appropriate
    /// location. This can be done for any Flutter application by
    /// running `flutter build bundle` while in the directory of a
    /// valid Flutter project. This should package all the code and
    /// assets in the "build/flutter_assets" directory. Specify this
    /// directory as the first argument to this utility.
    pub asset_bundle_path: PathBuf,
    /// Typically empty. These extra flags are passed directly to the
    /// Flutter engine. To see all supported flags, run
    /// `flutter_tester --help` using the test binary included in the
    /// Flutter tools.
    pub flutter_flags: Vec<String>,
}

fn main() -> Result<(), std::io::Error> {
    env_logger::init();
    let args = Args::parse();

    let event_loop: EventLoop<FlutterApplicationCallback> =
        EventLoopBuilder::with_user_event().build();
    let window = WindowBuilder::new()
        .with_title("Flutter Embedder")
        // .with_inner_size(PhysicalSize::new(1024, 768))
        .build(&event_loop)
        .unwrap();
    // window.set_outer_position(PhysicalPosition::new(100, 100));

    let rt = Arc::new(Builder::new_multi_thread().build()?);
    let inner_rt = rt.clone();

    rt.block_on(async move {
        let instance = Instance::new(Backends::VULKAN);
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    features: Features::CLEAR_TEXTURE,
                    limits: Limits::downlevel_defaults(),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let size = window.inner_size();

        log::debug!(
            "Supported formats: {:?}",
            surface.get_supported_formats(&adapter)
        );
        let formats = surface.get_supported_formats(&adapter);
        let format = formats
            .into_iter()
            .find(|&format| format == TextureFormat::Bgra8Unorm)
            .expect("Adapter doesn't support BGRA8 render buffer.");

        surface.configure(
            &device,
            &SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_DST,
                format,
                width: size.width,
                height: size.height,
                present_mode: PresentMode::Fifo,
            },
        );

        let window = Arc::new(window);
        let inner_window = window.clone();

        let mut app = FlutterApplication::new(
            inner_rt,
            &args.asset_bundle_path,
            args.flutter_flags,
            surface,
            Arc::new(instance),
            device,
            queue,
            event_loop.create_proxy(),
            window.clone(),
            move |cursor| {
                if let Some(cursor) = cursor {
                    inner_window.set_cursor_visible(true);
                    inner_window.set_cursor_icon(cursor);
                } else {
                    inner_window.set_cursor_visible(false);
                }
            },
        );

        app.register_platform_view_type("test", |data| Some(Box::new(TestPlatformView::new(data))));

        app.run();

        // Trigger a FlutterEngineSendWindowMetricsEvent to communicate the initial
        // size of the window.
        metrics_changed(&app, &window);

        event_loop.run(move |event, _, control_flow| {
            let _ = &adapter;

            *control_flow = ControlFlow::Wait;
            match event {
                Event::UserEvent(handler) => {
                    if handler(&mut app) {
                        *control_flow = ControlFlow::Exit;
                    }
                }
                Event::RedrawRequested(_window_id) => {
                    app.schedule_frame();
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::Moved(_)
                    | WindowEvent::Resized(_)
                    | WindowEvent::ScaleFactorChanged { .. } => {
                        metrics_changed(&app, &window);
                    }
                    WindowEvent::MouseInput {
                        device_id,
                        state,
                        button,
                        ..
                    } => {
                        app.mouse_buttons(device_id, state, button);
                    }
                    WindowEvent::CursorEntered { device_id } => {
                        app.mouse_entered(device_id);
                    }
                    WindowEvent::CursorLeft { device_id } => {
                        app.mouse_left(device_id);
                    }
                    WindowEvent::CursorMoved {
                        device_id,
                        position,
                        ..
                    } => {
                        app.mouse_moved(device_id, position);
                    }
                    WindowEvent::MouseWheel {
                        device_id,
                        delta,
                        phase,
                        ..
                    } => {
                        app.mouse_wheel(device_id, delta, phase);
                    }
                    WindowEvent::ModifiersChanged(state) => {
                        app.modifiers_changed(state);
                    }
                    WindowEvent::KeyboardInput {
                        event,
                        device_id,
                        is_synthetic,
                    } => {
                        app.key_event(device_id, event, is_synthetic);
                    }
                    WindowEvent::Focused(focused) => {
                        app.focused(focused);
                    }
                    _ => {}
                },
                _ => {}
            }
        });
    });
    Ok(())
}

fn metrics_changed(application: &FlutterApplication, window: &Window) {
    let size = window.inner_size();
    let position = window
        .inner_position()
        .unwrap_or(PhysicalPosition { x: 0, y: 0 });
    log::debug!(
        "scale_factor = {:?}",
        window.scale_factor(),
        // window
        //     .current_monitor()
        //     .map(|monitor| monitor.scale_factor())
    );
    application.metrics_changed(
        size.width,
        size.height,
        window
            .current_monitor()
            .map(|monitor| monitor.scale_factor())
            .unwrap_or(1.0),
        position.x,
        position.y,
    );
}
