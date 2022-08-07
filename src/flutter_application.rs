use std::{
    ffi::{CStr, CString},
    mem::{size_of, MaybeUninit},
    os::{
        raw::{c_char, c_void},
        unix::prelude::OsStrExt,
    },
    path::Path,
    ptr::{null, null_mut},
    sync::Mutex,
};

use ash::vk::Handle;
use wgpu::{Device, Instance, Queue};
use wgpu_hal::api::Vulkan;

use crate::{
    compositor::Compositor,
    flutter_bindings::{
        FlutterEngine, FlutterEngineInitialize, FlutterEngineResult,
        FlutterEngineResult_kInternalInconsistency, FlutterEngineResult_kInvalidArguments,
        FlutterEngineResult_kInvalidLibraryVersion, FlutterEngineResult_kSuccess,
        FlutterEngineRunInitialized, FlutterEngineSendPlatformMessageResponse,
        FlutterEngineSendWindowMetricsEvent, FlutterEngineShutdown, FlutterFrameInfo,
        FlutterPlatformMessage, FlutterProjectArgs, FlutterRendererConfig,
        FlutterRendererConfig__bindgen_ty_1, FlutterRendererType_kVulkan, FlutterVulkanImage,
        FlutterVulkanInstanceHandle, FlutterVulkanRendererConfig, FlutterWindowMetricsEvent,
        FLUTTER_ENGINE_VERSION,
    },
    utils::flutter_asset_bundle_is_valid,
};

pub struct FlutterApplication {
    engine: FlutterEngine,
    compositor: Mutex<Compositor>,
    instance: Instance,
    device: Device,
    queue: Queue,
}

impl FlutterApplication {
    pub fn new(
        asset_bundle_path: &Path,
        flutter_flags: Vec<String>,
        instance: Instance,
        device: Device,
        queue: Queue,
    ) -> Self {
        if !flutter_asset_bundle_is_valid(asset_bundle_path) {
            panic!("Flutter asset bundle was not valid.");
        }
        let icudtl_dat = Path::new("icudtl.dat");
        if icudtl_dat.exists() {
            panic!("icudtl.dat not found in the current directory.");
        }
        let (raw_instance, version, instance_extensions) = unsafe {
            instance.as_hal::<Vulkan, _, _>(|instance| {
                instance.map(|instance| {
                    let raw_instance = instance.shared_instance().raw_instance();
                    let raw_handle = raw_instance.handle().as_raw();

                    (
                        raw_handle,
                        instance.shared_instance().driver_api_version(),
                        instance
                            .shared_instance()
                            .extensions()
                            .into_iter()
                            .map(|&s| s.to_owned())
                            .collect::<Vec<CString>>(),
                    )
                })
            })
        }
        .expect("wgpu didn't choose Vulkan as rendering backend");

        let (raw_device, raw_physical_device, queue_family_index, raw_queue, device_extensions) =
            unsafe {
                device.as_hal::<Vulkan, _, _>(|device| {
                    device.map(|device| {
                        (
                            device.raw_device().handle().as_raw(),
                            device.raw_physical_device().as_raw(),
                            device.queue_family_index(),
                            device.raw_queue().as_raw(),
                            device
                                .enabled_device_extensions()
                                .into_iter()
                                .map(|&s| s.to_owned())
                                .collect::<Vec<CString>>(),
                        )
                    })
                })
            }
            .unwrap();

        let mut enabled_device_extensions: Vec<*const c_char> =
            device_extensions.iter().map(|ext| ext.as_ptr()).collect();
        let mut enabled_instance_extensions: Vec<*const c_char> =
            instance_extensions.iter().map(|ext| ext.as_ptr()).collect();

        let config = FlutterRendererConfig {
            type_: FlutterRendererType_kVulkan,
            __bindgen_anon_1: FlutterRendererConfig__bindgen_ty_1 {
                vulkan: FlutterVulkanRendererConfig {
                    struct_size: size_of::<FlutterVulkanRendererConfig>() as _,
                    version,
                    instance: raw_instance as _,
                    physical_device: raw_physical_device as _,
                    device: raw_device as _,
                    queue_family_index,
                    queue: raw_queue as _,
                    enabled_instance_extension_count: enabled_instance_extensions.len() as _,
                    enabled_instance_extensions: enabled_instance_extensions.as_mut_ptr(),
                    enabled_device_extension_count: enabled_device_extensions.len() as _,
                    enabled_device_extensions: enabled_device_extensions.as_mut_ptr(),
                    get_instance_proc_address_callback: Some(Self::instance_proc_address_callback),
                    get_next_image_callback: Some(Self::next_image),
                    present_image_callback: Some(Self::present_image),
                },
            },
        };

        let argv: Vec<CString> = flutter_flags
            .iter()
            .map(|arg| CString::new(arg.as_bytes()).unwrap())
            .collect();
        let argv_ptr: Vec<*const c_char> = argv
            .iter()
            .map(|arg| arg.as_bytes().as_ptr() as _)
            .collect();

        let compositor = Mutex::new(Compositor::new());
        let mut instance = Self {
            engine: null_mut(),
            compositor,
            instance,
            device,
            queue,
        };
        let flutter_compositor = instance
            .compositor
            .lock()
            .unwrap()
            .flutter_compositor(&instance);

        let mut args = unsafe { MaybeUninit::<FlutterProjectArgs>::zeroed().assume_init() };
        args.struct_size = size_of::<FlutterProjectArgs>() as _;
        args.assets_path = asset_bundle_path.as_os_str().as_bytes().as_ptr() as _;
        args.icu_data_path = icudtl_dat.as_os_str().as_bytes().as_ptr() as _;
        args.command_line_argc = flutter_flags.len() as _;
        args.command_line_argv = argv_ptr.as_ptr();
        args.platform_message_callback = Some(Self::platform_message_callback);
        // args.root_isolate_create_callback = todo!();
        // args.update_semantics_node_callback = todo!();
        // args.update_semantics_custom_action_callback = todo!();
        // args.vsync_callback = todo!();
        args.shutdown_dart_vm_when_done = true;
        args.compositor = &flutter_compositor as _;
        args.dart_old_gen_heap_size = -1;
        args.log_message_callback = Some(Self::log_message_callback);
        args.on_pre_engine_restart_callback = Some(Self::on_pre_engine_restart_callback);

        let mut engine = null_mut();

        Self::unwrap_result(unsafe {
            FlutterEngineInitialize(
                FLUTTER_ENGINE_VERSION.into(),
                &config as _,
                &args as _,
                &mut instance as *mut Self as _,
                &mut engine,
            )
        });

        drop(enabled_device_extensions);
        drop(enabled_instance_extensions);
        drop(instance_extensions);
        drop(device_extensions);
        drop(flutter_compositor);

        instance.engine = engine;

        instance
    }

    pub fn run(&self) {
        Self::unwrap_result(unsafe { FlutterEngineRunInitialized(self.engine) });
    }

    pub fn metrics_changed(&self, width: u32, height: u32, pixel_ratio: f64, x: i32, y: i32) {
        Self::unwrap_result(unsafe {
            FlutterEngineSendWindowMetricsEvent(
                self.engine,
                &FlutterWindowMetricsEvent {
                    struct_size: size_of::<FlutterWindowMetricsEvent>() as _,
                    width: width as _,
                    height: height as _,
                    pixel_ratio,
                    left: x.max(0) as _,
                    top: y.max(0) as _,
                    physical_view_inset_top: 0.0,
                    physical_view_inset_right: 0.0,
                    physical_view_inset_bottom: 0.0,
                    physical_view_inset_left: 0.0,
                },
            )
        });
    }

    pub fn instance(&self) -> &Instance {
        &self.instance
    }
    pub fn device(&self) -> &Device {
        &self.device
    }
    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    extern "C" fn platform_message_callback(
        message: *const FlutterPlatformMessage,
        user_data: *mut c_void,
    ) {
        let this = user_data as *mut Self;
        unsafe {
            log::debug!(
                "Platform message: channel = {}, message size = {}, message: {:?}",
                CStr::from_ptr((*message).channel).to_str().unwrap(),
                (*message).message_size,
                std::slice::from_raw_parts((*message).message, (*message).message_size as _),
            );
        }
        Self::unwrap_result(unsafe {
            FlutterEngineSendPlatformMessageResponse(
                (*this).engine,
                (*message).response_handle,
                null(),
                0,
            )
        });
    }

    extern "C" fn on_pre_engine_restart_callback(user_data: *mut c_void) {
        let _this = user_data as *mut Self;
        todo!()
    }

    extern "C" fn log_message_callback(
        tag: *const c_char,
        message: *const c_char,
        _user_data: *mut c_void,
    ) {
        let tag = unsafe { CStr::from_ptr(tag) };
        let message = unsafe { CStr::from_ptr(message) };
        log::logger().log(
            &log::Record::builder()
                .module_path(tag.to_str().ok())
                .args(format_args!("{}", message.to_str().unwrap()))
                .build(),
        );
    }

    extern "C" fn instance_proc_address_callback(
        user_data: *mut c_void,
        _instance: FlutterVulkanInstanceHandle,
        name: *const c_char,
    ) -> *mut c_void {
        let this = user_data as *mut Self;
        unsafe {
            (*this).instance.as_hal::<Vulkan, _, _>(|instance| {
                instance.and_then(|instance| {
                    let shared = instance.shared_instance();
                    let entry = shared.entry();
                    entry
                        .get_instance_proc_addr(shared.raw_instance().handle(), name)
                        .map(|f| f as *mut c_void)
                })
            })
        }
        .unwrap_or_else(null_mut)
    }

    extern "C" fn next_image(
        _user_data: *mut c_void,
        _frame_info: *const FlutterFrameInfo,
    ) -> FlutterVulkanImage {
        todo!()
        // Not used if a FlutterCompositor is supplied in FlutterProjectArgs.
    }

    extern "C" fn present_image(
        _user_data: *mut c_void,
        _image: *const FlutterVulkanImage,
    ) -> bool {
        todo!()
        // Not used if a FlutterCompositor is supplied in FlutterProjectArgs.
    }

    fn unwrap_result(result: FlutterEngineResult) {
        #[allow(non_upper_case_globals)]
        match result {
            x if x == FlutterEngineResult_kSuccess => {}
            x if x == FlutterEngineResult_kInvalidLibraryVersion => {
                panic!("Invalid library version.");
            }
            x if x == FlutterEngineResult_kInvalidArguments => {
                panic!("Invalid arguments.");
            }
            x if x == FlutterEngineResult_kInternalInconsistency => {
                panic!("Internal inconsistency.");
            }
            x => {
                panic!("Unknown error {x}.");
            }
        }
    }
}

impl Drop for FlutterApplication {
    fn drop(&mut self) {
        Self::unwrap_result(unsafe { FlutterEngineShutdown(self.engine) });
    }
}
