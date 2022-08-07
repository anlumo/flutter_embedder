use std::{ffi::c_void, mem::size_of, ptr::null_mut};

use ash::vk::Handle;
use wgpu::{Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};
use wgpu_hal::api::Vulkan;

use crate::{
    flutter_application::FlutterApplication,
    flutter_bindings::{
        size_t, FlutterBackingStore, FlutterBackingStoreConfig,
        FlutterBackingStoreType_kFlutterBackingStoreTypeVulkan, FlutterBackingStore__bindgen_ty_1,
        FlutterCompositor, FlutterLayer,
        FlutterLayerContentType_kFlutterLayerContentTypeBackingStore,
        FlutterLayerContentType_kFlutterLayerContentTypePlatformView, FlutterVulkanBackingStore,
        FlutterVulkanImage,
    },
};

pub struct Compositor {}

impl Compositor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn flutter_compositor(&self, application: &FlutterApplication) -> FlutterCompositor {
        FlutterCompositor {
            struct_size: size_of::<FlutterCompositor>() as _,
            user_data: application as *const FlutterApplication as _,
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
        let application =
            unsafe { &*(user_data as *const FlutterApplication) as &FlutterApplication };

        let texture = Box::new(application.device().create_texture(&TextureDescriptor {
            label: Some("Flutter Backing Store"),
            size: wgpu::Extent3d {
                width: unsafe { *config }.size.width as _,
                height: unsafe { *config }.size.height as _,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::COPY_SRC
                | TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::TEXTURE_BINDING,
        }));

        let mut image = None;
        unsafe {
            texture.as_hal::<Vulkan, _>(|texture| {
                let texture = texture.unwrap();
                image = Some(FlutterVulkanImage {
                    struct_size: size_of::<FlutterVulkanImage>() as _,
                    image: texture.raw_handle().as_raw() as _,
                    format: ash::vk::Format::R8G8B8A8_UNORM.as_raw() as _,
                });
            });
        }
        let mut backing_store = unsafe { &mut *backing_store_out as &mut FlutterBackingStore };
        backing_store.user_data = null_mut();
        backing_store.type_ = FlutterBackingStoreType_kFlutterBackingStoreTypeVulkan;
        backing_store.did_update = true;
        backing_store.__bindgen_anon_1 = FlutterBackingStore__bindgen_ty_1 {
            vulkan: FlutterVulkanBackingStore {
                struct_size: size_of::<FlutterVulkanBackingStore>() as _,
                image: &image.unwrap(),
                user_data: Box::into_raw(texture) as _,
                destruction_callback: Some(Self::destroy_texture),
            },
        };
        true
    }
    extern "C" fn destroy_texture(user_data: *mut c_void) {
        let texture = unsafe { Box::from_raw(user_data as *mut Texture) };
        texture.destroy();
    }
    extern "C" fn present_layers_callback(
        layers: *mut *const FlutterLayer,
        layers_count: size_t,
        user_data: *mut c_void,
    ) -> bool {
        let _this = user_data as *const FlutterApplication;
        for &layer in unsafe { std::slice::from_raw_parts(layers, layers_count as _) } {
            let layer = unsafe { &*layer };
            let offset = layer.offset;
            let size = layer.size;
            match layer.type_ {
                x if x == FlutterLayerContentType_kFlutterLayerContentTypeBackingStore => {
                    let backing_store = unsafe { &*layer.__bindgen_anon_1.backing_store };
                    assert_eq!(
                        backing_store.type_,
                        FlutterBackingStoreType_kFlutterBackingStoreTypeVulkan
                    );
                    let backing_store = unsafe { &backing_store.__bindgen_anon_1.vulkan };
                    let texture = unsafe { &*(backing_store.user_data as *mut Texture) };

                    // TODO: draw texture to backing buffer
                    todo!()
                }
                x if x == FlutterLayerContentType_kFlutterLayerContentTypePlatformView => todo!(),
                _ => panic!("Invalid layer type"),
            }
        }

        todo!()
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
