use std::{cell::Cell, ffi::c_void, mem::size_of, ptr::null_mut};

use ash::vk::Handle;
use wgpu::{
    include_wgsl, Color, CommandEncoderDescriptor, LoadOp, Operations, PresentMode,
    RenderPassColorAttachment, RenderPassDescriptor, SurfaceConfiguration, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages,
};
use wgpu_hal::api::Vulkan;

use crate::{
    flutter_application::FlutterApplication,
    flutter_bindings::{
        size_t, FlutterBackingStore, FlutterBackingStoreConfig,
        FlutterBackingStoreType_kFlutterBackingStoreTypeVulkan, FlutterBackingStore__bindgen_ty_1,
        FlutterCompositor, FlutterLayer,
        FlutterLayerContentType_kFlutterLayerContentTypeBackingStore,
        FlutterLayerContentType_kFlutterLayerContentTypePlatformView, FlutterRect,
        FlutterRoundedRect, FlutterTransformation, FlutterVulkanBackingStore, FlutterVulkanImage,
    },
};

use super::FlutterApplicationUserData;

#[derive(Debug, Clone)]
pub enum PlatformViewMutation {
    /// Indicates that the Flutter application requested that an opacity be
    /// applied to the platform view.
    Opacity(f64),
    /// Indicates that the Flutter application requested that the platform view be
    /// clipped using a rectangle.
    ClipRect(FlutterRect),
    /// Indicates that the Flutter application requested that the platform view be
    /// clipped using a rounded rectangle.
    ClipRoundedRect(FlutterRoundedRect),
    /// Indicates that the Flutter application requested that the platform view be
    /// transformed before composition.
    Transformation(FlutterTransformation),
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct FlutterRenderUniform {
    offset: [f32; 2],
    size: [f32; 2],
    viewport: [f32; 2],
}

struct CompositorBackingBufferInformation {
    texture_bind_group: wgpu::BindGroup,
    uniform_bind_group: wgpu::BindGroup,
    image: FlutterVulkanImage,
    uniform_buffer: wgpu::Buffer,
}

pub struct Compositor {
    render_pipeline: wgpu::RenderPipeline,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    uniform_bind_group_layout: wgpu::BindGroupLayout,
    previous_viewport_size: Cell<(u32, u32)>,
}

impl Compositor {
    pub fn new(device: &wgpu::Device, viewport_size: (u32, u32)) -> Self {
        let shader = device.create_shader_module(include_wgsl!("flutter.wgsl"));
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("Compositor Texture Bind Group Layout"),
            });
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("Compositor Uniform Bind Group Layout"),
            });
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Compositor Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout],
                push_constant_ranges: &[],
            });
        Self {
            render_pipeline: device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Compositor Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: TextureFormat::Bgra8Unorm,
                        blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            }),
            texture_bind_group_layout,
            uniform_bind_group_layout,
            previous_viewport_size: Cell::new(viewport_size),
        }
    }

    pub fn flutter_compositor(application: &FlutterApplication) -> FlutterCompositor {
        FlutterCompositor {
            struct_size: size_of::<FlutterCompositor>() as _,
            user_data: &*application.user_data as *const FlutterApplicationUserData as _,
            create_backing_store_callback: Some(Self::create_backing_store_callback),
            collect_backing_store_callback: Some(Self::backing_store_collect_callback),
            present_layers_callback: Some(Self::present_layers_callback),
            avoid_backing_store_cache: false,
        }
    }

    extern "C" fn create_backing_store_callback(
        config: *const FlutterBackingStoreConfig,
        backing_store_out: *mut FlutterBackingStore,
        user_data: *mut c_void,
    ) -> bool {
        let application_user_data = unsafe {
            &*(user_data as *const FlutterApplicationUserData) as &FlutterApplicationUserData
        };

        let device = &application_user_data.device;

        let texture = device.create_texture(&TextureDescriptor {
            label: Some("Flutter Backing Store"),
            size: wgpu::Extent3d {
                width: unsafe { *config }.size.width as _,
                height: unsafe { *config }.size.height as _,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8Unorm,
            usage: TextureUsages::COPY_SRC
                | TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::TEXTURE_BINDING,
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &application_user_data.compositor.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Compositor Bind Group"),
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Compositor Uniform Buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: size_of::<FlutterRenderUniform>() as u64,
            mapped_at_creation: false,
        });
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compositor Uniform Bind Group"),
            layout: &application_user_data.compositor.uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let mut image = None;
        unsafe {
            texture.as_hal::<Vulkan, _>(|texture| {
                let texture = texture.unwrap();
                image = Some(FlutterVulkanImage {
                    struct_size: size_of::<FlutterVulkanImage>() as _,
                    image: texture.raw_handle().as_raw() as _,
                    format: ash::vk::Format::B8G8R8A8_UNORM.as_raw() as _,
                });
            });
        }

        let image = image.unwrap();
        let user_data = Box::new(CompositorBackingBufferInformation {
            texture_bind_group,
            uniform_bind_group,
            image,
            uniform_buffer,
        });
        let mut backing_store = unsafe { &mut *backing_store_out as &mut FlutterBackingStore };
        backing_store.user_data = null_mut();
        backing_store.type_ = FlutterBackingStoreType_kFlutterBackingStoreTypeVulkan;
        backing_store.did_update = true;
        backing_store.__bindgen_anon_1 = FlutterBackingStore__bindgen_ty_1 {
            vulkan: FlutterVulkanBackingStore {
                struct_size: size_of::<FlutterVulkanBackingStore>() as _,
                image: &user_data.image,
                user_data: Box::into_raw(user_data) as _,
                destruction_callback: Some(Self::destroy_texture),
            },
        };
        true
    }
    extern "C" fn destroy_texture(user_data: *mut c_void) {
        let _ = *unsafe { Box::from_raw(user_data as *mut CompositorBackingBufferInformation) };
    }
    extern "C" fn present_layers_callback(
        layers: *mut *const FlutterLayer,
        layers_count: size_t,
        user_data: *mut c_void,
    ) -> bool {
        let application_user_data = unsafe { &*(user_data as *const FlutterApplicationUserData) };

        let viewport_size = application_user_data.viewport_size.get();
        if viewport_size
            != application_user_data
                .compositor
                .previous_viewport_size
                .get()
        {
            application_user_data.surface.configure(
                &application_user_data.device,
                &SurfaceConfiguration {
                    usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_DST,
                    format: TextureFormat::Bgra8Unorm,
                    width: viewport_size.0 as _,
                    height: viewport_size.1 as _,
                    present_mode: PresentMode::Fifo,
                },
            );
            application_user_data
                .compositor
                .previous_viewport_size
                .set(viewport_size);
        }

        let frame = application_user_data
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let mut encoder = application_user_data
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: None });
        {
            let view = frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&application_user_data.compositor.render_pipeline);

            let mut platform_views_handler =
                application_user_data.platform_views_handler.lock().unwrap();

            let layers = unsafe { std::slice::from_raw_parts(layers, layers_count as _) };

            let viewport_size = application_user_data.viewport_size.get();
            let uniform_buffers: Vec<_> = layers
                .iter()
                .map(|layer| {
                    let layer = unsafe { &**layer };
                    if layer.type_ == FlutterLayerContentType_kFlutterLayerContentTypeBackingStore {
                        bytemuck::cast_slice(&[FlutterRenderUniform {
                            offset: [layer.offset.x as f32, layer.offset.y as f32],
                            size: [layer.size.width as f32, layer.size.height as f32],
                            viewport: [viewport_size.0 as _, viewport_size.1 as _],
                        }])
                        .to_vec()
                    } else {
                        bytemuck::cast_slice(&[FlutterRenderUniform::default()]).to_vec()
                    }
                })
                .collect();

            for (idx, &layer) in layers
                .iter()
                .map(|&layer| unsafe { &*layer } as &FlutterLayer)
                .enumerate()
            {
                let offset = layer.offset;
                let size = layer.size;
                log::debug!(
                    "Layer {idx} type {} offset {offset:?} size {size:?}",
                    layer.type_
                );
                match layer.type_ {
                    x if x == FlutterLayerContentType_kFlutterLayerContentTypeBackingStore => {
                        let backing_store = unsafe { &*layer.__bindgen_anon_1.backing_store };
                        assert_eq!(
                            backing_store.type_,
                            FlutterBackingStoreType_kFlutterBackingStoreTypeVulkan
                        );
                        let backing_store = unsafe { &backing_store.__bindgen_anon_1.vulkan };
                        let information = &unsafe {
                            &*(backing_store.user_data as *const CompositorBackingBufferInformation)
                        };

                        application_user_data.queue.write_buffer(
                            &information.uniform_buffer,
                            0,
                            &uniform_buffers[idx],
                        );

                        render_pass.set_bind_group(0, &information.texture_bind_group, &[]);
                        render_pass.set_bind_group(1, &information.uniform_bind_group, &[]);
                        render_pass.draw(0..4, 0..1);
                    }
                    x if x == FlutterLayerContentType_kFlutterLayerContentTypePlatformView => {
                        let platform_view = unsafe { &*layer.__bindgen_anon_1.platform_view };
                        platform_views_handler.render_platform_view(
                            platform_view.identifier as i32,
                            (offset.x, offset.y),
                            (size.width, size.height),
                            unsafe {
                                std::slice::from_raw_parts(
                                    platform_view.mutations,
                                    platform_view.mutations_count as usize,
                                )
                            },
                        );
                    }
                    _ => panic!("Invalid layer type"),
                }
            }
        }
        application_user_data.queue.submit(Some(encoder.finish()));
        frame.present();
        true
    }
    extern "C" fn backing_store_collect_callback(
        _renderer: *const FlutterBackingStore,
        _user_data: *mut c_void,
    ) -> bool {
        // let _this = user_data as *const FlutterApplication;
        // destroy the user_data in FlutterBackingStore. Since we passed nullptr there, there's nothing to do
        true
    }
}
