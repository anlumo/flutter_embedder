#![allow(dead_code)]
use std::path::PathBuf;

use clap::Parser;
use wgpu::{
    Backends, DeviceDescriptor, Features, Instance, Limits, PowerPreference, PresentMode,
    RequestAdapterOptions, SurfaceConfiguration, TextureUsages,
};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

mod flutter_application;
use flutter_application::FlutterApplication;
mod compositor;

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

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::parse();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Flutter Embedder")
        // .with_inner_size(PhysicalSize::new(1024, 768))
        .build(&event_loop)
        .unwrap();
    // window.set_outer_position(PhysicalPosition::new(100, 100));

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
                features: Features::empty(),
                limits: Limits::downlevel_defaults(),
            },
            None,
        )
        .await
        .expect("Failed to create device");

    let size = window.inner_size();

    surface.configure(
        &device,
        &SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
        },
    );

    let flutter = FlutterApplication::new(
        &args.asset_bundle_path,
        args.flutter_flags,
        surface,
        instance,
        device,
        queue,
    );

    flutter.run();

    // Trigger a FlutterEngineSendWindowMetricsEvent to communicate the initial
    // size of the window.
    metrics_changed(&flutter, &window);

    event_loop.run(move |event, _, control_flow| {
        let _ = &adapter;

        *control_flow = ControlFlow::Wait;
        match event {
            Event::RedrawRequested(_window_id) => {
                flutter.schedule_frame();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event:
                    WindowEvent::Moved(_)
                    | WindowEvent::Resized(_)
                    | WindowEvent::ScaleFactorChanged { .. },
                ..
            } => {
                metrics_changed(&flutter, &window);
            }
            _ => {}
        }
    });
}

fn metrics_changed(flutter: &FlutterApplication, window: &Window) {
    let size = window.inner_size();
    let position = window
        .inner_position()
        .unwrap_or(PhysicalPosition { x: 0, y: 0 });
    flutter.metrics_changed(
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
