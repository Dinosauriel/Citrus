use ash::vk::{self};
use super::graphics_state::GraphicState;

#[derive(Clone, Copy)]
pub enum PipelineType {
    World,
    WorldLine,
    Hud,
}

// holds on to all necessary information for the pipeline to live
pub struct Pipeline<'a> {
    rasterization_info: vk::PipelineRasterizationStateCreateInfo<'a>,
    dynamic_state: vk::PipelineDynamicStateCreateInfo<'a>,
    input_assembly_state: vk::PipelineInputAssemblyStateCreateInfo<'a>,
    viewport_state: vk::PipelineViewportStateCreateInfo<'a>,
    multisample_state: vk::PipelineMultisampleStateCreateInfo<'a>,
    depth_stencil_state: vk::PipelineDepthStencilStateCreateInfo<'a>,
    color_blend_state: vk::PipelineColorBlendStateCreateInfo<'a>,
}

impl<'a> Pipeline<'a> {
    pub unsafe fn new(p_type: PipelineType, scissors: &'a [vk::Rect2D; 1], viewports: &'a [vk::Viewport; 1]) -> Self {
        Pipeline {
            rasterization_info: match p_type {
                PipelineType::World => rasterization_info_fill(),
                PipelineType::WorldLine => rasterization_info_line(),
                PipelineType::Hud => rasterization_info_fill(),
            },
            dynamic_state: dynamic_state_create_info(),
            input_assembly_state: input_assembly_state(),
            viewport_state: viewport_state_create_info(&scissors, &viewports),
            multisample_state: multisample_state_create_info(),
            depth_stencil_state: depth_stencil_state_create_info(),
            color_blend_state: color_blend_state_create_info(),
        }
    }


    pub unsafe fn create_info(&'a self,
        shader_stages: &'a [vk::PipelineShaderStageCreateInfo; 2], vertex_input_state: &'a vk::PipelineVertexInputStateCreateInfo,
        render_pass: vk::RenderPass, pipeline_layout: vk::PipelineLayout) -> vk::GraphicsPipelineCreateInfo<'a> {

        vk::GraphicsPipelineCreateInfo::default()
            .stages(shader_stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&self.input_assembly_state)
            .viewport_state(&self.viewport_state)
            .rasterization_state(&self.rasterization_info)
            .multisample_state(&self.multisample_state)
            .depth_stencil_state(&self.depth_stencil_state)
            .color_blend_state(&self.color_blend_state)
            .dynamic_state(&self.dynamic_state)
            .render_pass(render_pass)
            .layout(pipeline_layout)
    }
}

pub fn render_pass(g_state: &GraphicState) -> vk::RenderPass {
    let renderpass_attachments = [
        vk::AttachmentDescription {
            format: g_state.surface_format.format,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            ..Default::default()
        },
        vk::AttachmentDescription {
            format: vk::Format::D16_UNORM,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            initial_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            ..Default::default()
        },
    ];
    let color_attachment_refs = [vk::AttachmentReference {
        attachment: 0,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    }];
    let depth_attachment_ref = vk::AttachmentReference {
        attachment: 1,
        layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
    };
    let dependencies = [vk::SubpassDependency {
        src_subpass: vk::SUBPASS_EXTERNAL,
        src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ
            | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        ..Default::default()
    }];

    let subpass = vk::SubpassDescription::default()
        .color_attachments(&color_attachment_refs)
        .depth_stencil_attachment(&depth_attachment_ref)
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS);

    let renderpass_create_info = vk::RenderPassCreateInfo::default()
        .attachments(&renderpass_attachments)
        .subpasses(std::slice::from_ref(&subpass))
        .dependencies(&dependencies);

    unsafe {
        g_state.device.create_render_pass(&renderpass_create_info, None).unwrap()
    }
}

pub fn scissors(g_state: &GraphicState) -> [vk::Rect2D; 1] {
    [vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: g_state.surface_resolution,
    }]
}

pub fn viewports(g_state: &GraphicState) -> [vk::Viewport; 1] {
    [vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: g_state.surface_resolution.width as f32,
        height: g_state.surface_resolution.height as f32,
        min_depth: 0.0,
        max_depth: 1.0,
    }]
}

fn viewport_state_create_info<'a>(scissors: &'a [vk::Rect2D; 1], viewports: &'a [vk::Viewport; 1]) 
        -> vk::PipelineViewportStateCreateInfo<'a> {
    vk::PipelineViewportStateCreateInfo::default()
        .viewports(viewports)
        .scissors(scissors)
}


fn multisample_state_create_info() -> vk::PipelineMultisampleStateCreateInfo<'static> {
    vk::PipelineMultisampleStateCreateInfo {
        rasterization_samples: vk::SampleCountFlags::TYPE_1,
        ..Default::default()
    }
}

fn depth_stencil_state_create_info() -> vk::PipelineDepthStencilStateCreateInfo<'static> {
    let noop_stencil_state = vk::StencilOpState {
        fail_op: vk::StencilOp::KEEP,
        pass_op: vk::StencilOp::KEEP,
        depth_fail_op: vk::StencilOp::KEEP,
        compare_op: vk::CompareOp::ALWAYS,
        ..Default::default()
    };

    vk::PipelineDepthStencilStateCreateInfo {
        depth_test_enable: vk::TRUE,
        depth_write_enable: vk::TRUE,
        depth_compare_op: vk::CompareOp::LESS_OR_EQUAL,
        front: noop_stencil_state,
        back: noop_stencil_state,
        max_depth_bounds: 1.0,
        ..Default::default()
    }
}

static COLOR_BLEND_ATTACHMENT_STATES: [vk::PipelineColorBlendAttachmentState; 1] = [vk::PipelineColorBlendAttachmentState {
    blend_enable: vk::TRUE,
    src_color_blend_factor: vk::BlendFactor::SRC_ALPHA,
    dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_DST_ALPHA,
    color_blend_op: vk::BlendOp::ADD,
    src_alpha_blend_factor: vk::BlendFactor::SRC_ALPHA,
    dst_alpha_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
    alpha_blend_op: vk::BlendOp::ADD,
    color_write_mask: vk::ColorComponentFlags::RGBA,
}];

fn color_blend_state_create_info() -> vk::PipelineColorBlendStateCreateInfo<'static> {
    vk::PipelineColorBlendStateCreateInfo::default()
        .logic_op(vk::LogicOp::CLEAR)
        .attachments(&COLOR_BLEND_ATTACHMENT_STATES)
}

fn rasterization_info_fill() -> vk::PipelineRasterizationStateCreateInfo<'static> {
    vk::PipelineRasterizationStateCreateInfo {
        front_face: vk::FrontFace::COUNTER_CLOCKWISE,
        line_width: 1.0,
        polygon_mode: vk::PolygonMode::FILL,
        cull_mode: vk::CullModeFlags::BACK,
        ..Default::default()
    }
}

fn rasterization_info_line() -> vk::PipelineRasterizationStateCreateInfo<'static> {
    vk::PipelineRasterizationStateCreateInfo {
        front_face: vk::FrontFace::COUNTER_CLOCKWISE,
        line_width: 1.0,
        polygon_mode: vk::PolygonMode::LINE,
        cull_mode: vk::CullModeFlags::BACK,
        ..Default::default()
    }
}

fn dynamic_state_create_info() -> vk::PipelineDynamicStateCreateInfo<'static> {
    vk::PipelineDynamicStateCreateInfo::default()
        .dynamic_states(&[vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR])
}

pub fn vertex_input_state<'a>(bindigs: &'a [vk::VertexInputBindingDescription; 1], attrs: &'a [vk::VertexInputAttributeDescription; 2])
        -> vk::PipelineVertexInputStateCreateInfo<'a> {
    vk::PipelineVertexInputStateCreateInfo::default()
        .vertex_attribute_descriptions(attrs)
        .vertex_binding_descriptions(bindigs)
}

fn input_assembly_state() -> vk::PipelineInputAssemblyStateCreateInfo<'static> {
    vk::PipelineInputAssemblyStateCreateInfo {
        topology: vk::PrimitiveTopology::TRIANGLE_LIST,
        ..Default::default()
    }
}

pub unsafe fn pipeline_layout(g_state: &GraphicState, descriptor_set_layout: &vk::DescriptorSetLayout) 
        -> vk::PipelineLayout {
    let layout_create_info = vk::PipelineLayoutCreateInfo {
        set_layout_count: 1,
        p_set_layouts: descriptor_set_layout,
        ..Default::default()
    };
    g_state.device.create_pipeline_layout(&layout_create_info, None).unwrap()
}
