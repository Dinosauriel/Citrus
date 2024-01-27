use std::io::Cursor;
use ash::vk;
use ash::util::*;

pub unsafe fn get_shader_module(spv_file: &mut Cursor<&[u8]>, device: &ash::Device) -> vk::ShaderModule {
    let code = read_spv(spv_file).expect("failed to read shader spv file");
    let shader_info = vk::ShaderModuleCreateInfo::builder().code(&code);

    let shader_module = device.create_shader_module(&shader_info, None).expect("shader module error");
    return shader_module;
}

pub unsafe fn get_shader_stage_create_infos(vertex_shader: vk::ShaderModule, fragment_shader: vk::ShaderModule) -> [vk::PipelineShaderStageCreateInfo; 2] {
    let main_str = std::ffi::CStr::from_bytes_with_nul_unchecked(b"main\0");

    [vk::PipelineShaderStageCreateInfo {
        module: vertex_shader,
        p_name: main_str.as_ptr(),
        stage: vk::ShaderStageFlags::VERTEX,
        ..Default::default()
    },
    vk::PipelineShaderStageCreateInfo {
        module: fragment_shader,
        p_name: main_str.as_ptr(),
        stage: vk::ShaderStageFlags::FRAGMENT,
        ..Default::default()
    }]
}