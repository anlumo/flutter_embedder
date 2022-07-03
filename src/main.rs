use std::path::PathBuf;

use clap::Parser;
use wgpu::{
    Backends, Color, CommandEncoderDescriptor, DeviceDescriptor, Features, Instance, Limits,
    LoadOp, Operations, PresentMode, RenderPassColorAttachment, RenderPassDescriptor,
    RequestAdapterOptions, SurfaceConfiguration, TextureUsages, TextureViewDescriptor,
};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod flutter_application;
use flutter_application::FlutterApplication;

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
        .with_inner_size(PhysicalSize::new(1024, 768))
        .build(&event_loop)
        .unwrap();
    window.set_outer_position(PhysicalPosition::new(100, 100));

    let instance = Instance::new(Backends::VULKAN);
    let surface = unsafe { instance.create_surface(&window) };
    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
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

    surface.configure(
        &device,
        &SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: 1024,
            height: 768,
            present_mode: PresentMode::Fifo,
        },
    );

    let flutter = FlutterApplication::new(
        &args.asset_bundle_path,
        args.flutter_flags,
        &instance,
        &device,
        &queue,
    );

    event_loop.run(move |event, _, control_flow| {
        let _ = (&instance, &adapter);

        *control_flow = ControlFlow::Wait;
        match event {
            Event::RedrawRequested(_window_id) => {
                let frame = surface
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture");
                let view = frame.texture.create_view(&TextureViewDescriptor::default());
                let mut encoder =
                    device.create_command_encoder(&CommandEncoderDescriptor { label: None });
                {
                    encoder.begin_render_pass(&RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Clear(Color {
                                    r: 0.0,
                                    g: 1.0,
                                    b: 0.0,
                                    a: 1.0,
                                }),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                    });
                }
                queue.submit(Some(encoder.finish()));
                frame.present();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}
