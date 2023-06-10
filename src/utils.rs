use std::path::Path;

pub fn flutter_asset_bundle_is_valid(bundle_path: &Path) -> bool {
    if !bundle_path.exists() {
        log::error!("Bundle directory does not exist.");
        return false;
    }

    let mut kernel_path = bundle_path.to_path_buf();
    kernel_path.push("kernel_blob.bin");

    if !kernel_path.exists() {
        log::error!("Kernel blob {} does not exist.", kernel_path.display());
        return false;
    }
    true
}
