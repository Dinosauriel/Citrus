use std::io::Cursor;
use ash::vk;
use ash::util::*;

pub unsafe fn get_shader_module(spv_file: &mut Cursor<&[u8]>, device: &ash::Device) -> vk::ShaderModule {
    let code = read_spv(spv_file).expect("failed to read shader spv file");
    let shader_info = vk::ShaderModuleCreateInfo::builder().code(&code);

    let shader_module = device.create_shader_module(&shader_info, None).expect("shader module error");
    return shader_module;
}