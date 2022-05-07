use std::{
    ffi::{CStr, CString},
    mem::{size_of, MaybeUninit},
    os::{
        raw::{c_char, c_void},
        unix::prelude::OsStrExt,
    },
    path::Path,
    ptr::{null, null_mut},
};

use crate::{
    flutter_bindings::{
        size_t, FlutterBackingStore, FlutterBackingStoreConfig, FlutterCompositor,
        FlutterEngineResult_kInternalInconsistency, FlutterEngineResult_kInvalidArguments,
        FlutterEngineResult_kInvalidLibraryVersion, FlutterEngineResult_kSuccess, FlutterEngineRun,
        FlutterLayer, FlutterPlatformMessage, FlutterProjectArgs, FlutterRendererConfig,
        FlutterRendererConfig__bindgen_ty_1, FlutterRendererType_kVulkan,
        FlutterVulkanRendererConfig, FLUTTER_ENGINE_VERSION,
    },
    utils::flutter_asset_bundle_is_valid,
};

pub struct FlutterApplication {}

impl FlutterApplication {
    pub fn new(asset_bundle_path: &Path, flutter_flags: Vec<String>) -> Self {
        if !flutter_asset_bundle_is_valid(asset_bundle_path) {
            panic!("Flutter asset bundle was not valid.");
        }
        let icudtl_dat = Path::new("icudtl.dat");
        if icudtl_dat.exists() {
            panic!("icudtl.dat not found in the current directory.");
        }

        let config = FlutterRendererConfig {
            type_: FlutterRendererType_kVulkan,
            __bindgen_anon_1: FlutterRendererConfig__bindgen_ty_1 {
                vulkan: FlutterVulkanRendererConfig {
                    struct_size: size_of::<FlutterVulkanRendererConfig>() as _,
                    version: todo!(),
                    instance: todo!(),
                    physical_device: todo!(),
                    device: todo!(),
                    queue_family_index: todo!(),
                    queue: todo!(),
                    enabled_instance_extension_count: todo!(),
                    enabled_instance_extensions: todo!(),
                    enabled_device_extension_count: todo!(),
                    enabled_device_extensions: todo!(),
                    get_instance_proc_address_callback: todo!(),
                    get_next_image_callback: todo!(),
                    present_image_callback: todo!(),
                },
            },
        };

        let argv: Vec<CString> = flutter_flags
            .into_iter()
            .map(|arg| CString::new(arg).unwrap())
            .collect();
        let argv_ptr: Vec<*const c_char> = argv
            .iter()
            .map(|arg| arg.as_bytes().as_ptr() as _)
            .collect();

        let mut instance = Self {};
        let compositor = FlutterCompositor {
            struct_size: size_of::<FlutterCompositor>() as _,
            user_data: &mut instance as *mut Self as _,
            create_backing_store_callback: Some(Self::create_backing_store_callback),
            collect_backing_store_callback: Some(Self::backing_store_collect_callback),
            present_layers_callback: Some(Self::present_layers_callback),
            avoid_backing_store_cache: false,
        };

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
        args.compositor = &compositor as _;
        args.dart_old_gen_heap_size = -1;
        args.log_message_callback = Some(Self::log_message_callback);
        args.on_pre_engine_restart_callback = Some(Self::on_pre_engine_restart_callback);

        let mut engine = null_mut();

        #[allow(non_upper_case_globals)]
        match unsafe {
            FlutterEngineRun(
                FLUTTER_ENGINE_VERSION.into(),
                &config as _,
                &args as _,
                &mut instance as *mut Self as _,
                &mut engine,
            )
        } {
            FlutterEngineResult_kSuccess => instance,
            FlutterEngineResult_kInvalidLibraryVersion => {
                panic!("Invalid library version.");
            }
            FlutterEngineResult_kInvalidArguments => {
                panic!("Invalid arguments.");
            }
            FlutterEngineResult_kInternalInconsistency => {
                panic!("Internal inconsistency.");
            }
            x => {
                panic!("Unknown error {x}.");
            }
        }
    }
    extern "C" fn platform_message_callback(
        _message: *const FlutterPlatformMessage,
        user_data: *mut c_void,
    ) {
        let _this = user_data as *mut Self;
        todo!()
    }

    extern "C" fn create_backing_store_callback(
        _config: *const FlutterBackingStoreConfig,
        _backing_store_out: *mut FlutterBackingStore,
        user_data: *mut c_void,
    ) -> bool {
        let _this = user_data as *mut Self;
        todo!()
    }
    extern "C" fn present_layers_callback(
        _layers: *mut *const FlutterLayer,
        _layers_count: size_t,
        user_data: *mut c_void,
    ) -> bool {
        let _this = user_data as *mut Self;
        todo!()
    }
    extern "C" fn backing_store_collect_callback(
        _renderer: *const FlutterBackingStore,
        user_data: *mut c_void,
    ) -> bool {
        let _this = user_data as *mut Self;
        todo!()
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
}
