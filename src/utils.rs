use std::path::Path;

pub fn flutter_asset_bundle_is_valid(bundle_path: &Path) -> bool {
    if !bundle_path.exists() {
        log::error!("Bundle directory does not exist.");
        return false;
    }

    if !bundle_path.with_file_name("kernel_blob.bin").exists() {
        log::error!("Kernel blob does not exist.");
        return false;
    }
    return true;
}
