use ash::vk;
use ash::util::*;
use super::graphics_state::GraphicState;

pub enum ShaderType {
    World,
    Hud,
}

pub enum ShaderStage {
    Fragment,
    Vertex,
}

pub unsafe fn shader_module(g_state: &GraphicState, s_type: ShaderType, s_stage: ShaderStage) -> vk::ShaderModule {
    let mut spv = match (s_type, s_stage) {
        (ShaderType::World, ShaderStage::Fragment) => std::io::Cursor::new(&include_bytes!("../shaders/frag.spv")[..]),
        (ShaderType::World, ShaderStage::Vertex) => std::io::Cursor::new(&include_bytes!("../shaders/vert.spv")[..]),
        (ShaderType::Hud, ShaderStage::Fragment) => std::io::Cursor::new(&include_bytes!("../shaders/hud_frag.spv")[..]),
        (ShaderType::Hud, ShaderStage::Vertex) => std::io::Cursor::new(&include_bytes!("../shaders/hud_vert.spv")[..]),
    };

    let code = read_spv(&mut spv).expect("failed to read shader spv file");
    let shader_info = vk::ShaderModuleCreateInfo::default().code(&code);

    g_state.device.create_shader_module(&shader_info, None).expect("shader module error")
}

pub unsafe fn shader_stage_create_infos(vertex_shader: vk::ShaderModule, fragment_shader: vk::ShaderModule) -> [vk::PipelineShaderStageCreateInfo<'static>; 2] {
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