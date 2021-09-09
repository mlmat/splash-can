use std::ffi::{CStr, CString};
use ash::version::EntryV1_0;

const BUNDLED_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];

pub fn check_validation_layer_support(entry: &ash::Entry) -> bool {
    let layer_properties = entry
        .enumerate_instance_layer_properties()
        .expect("Failed to enumerate Instance Layers Properties!");

    if layer_properties.len() <= 0 {
        eprintln!("Layer properties not found!");
        return false;
    }

    for bundled_layer in BUNDLED_LAYERS.iter() {
        let mut layer_found = false;

        'inner: for property in layer_properties.iter() {
            let property_name = unsafe { 
                let pointer = property.layer_name.as_ptr();
                CStr::from_ptr(pointer)
            }.to_str()
            .expect("Failed vulkan raw string conversion!")
            .to_owned();
    
            if property_name == *bundled_layer {
                layer_found = true;
                break 'inner;
            } 
        }
        if layer_found == false {
            return false;
        }
    }
    true
}

pub fn get_validation_layers() -> Vec<*const i8> {
    let raw_names: Vec<CString> = BUNDLED_LAYERS
        .iter()
        .map(|layer_name| CString::new(*layer_name).unwrap())
        .collect();
    raw_names
        .iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect()
}