use ash::util::*;
use ash::vk;
use glam::{Mat4, Vec3};
use citrus::*;
use std::default::Default;
use std::ffi::CStr;
use std::io::Cursor;
use std::mem;
use std::mem::align_of;
use std::time;
use glfw::{Action, Context, Key};
use noise::{NoiseFn, Perlin};
use controls::InputState;

#[derive(Clone, Debug, Copy)]
struct Vertex {
    pos: [f32; 4],
    color: [f32; 4],
}

#[derive(Clone, Debug, Copy)]
#[allow(dead_code)]
struct UniformBufferObject {
    model: Mat4,
    view: Mat4,
    proj: Mat4,
}

unsafe fn get_shader_module(spv_file: &mut Cursor<&[u8]>, device: &ash::Device) -> vk::ShaderModule {
    let code = read_spv(spv_file).expect("failed to read shader spv file");
    let shader_info = vk::ShaderModuleCreateInfo::builder().code(&code);

    let shader_module = device.create_shader_module(&shader_info, None).expect("shader module error");
    return shader_module;
}

unsafe fn create_descriptor_set_layout(device: &ash::Device) -> ash::vk::DescriptorSetLayout {
    let ubo_layout_binding = vk::DescriptorSetLayoutBinding {
        binding: 0,
        descriptor_count: 1,
        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
        stage_flags: vk::ShaderStageFlags::VERTEX,
        ..Default::default()
    };

    let layout_info = vk::DescriptorSetLayoutCreateInfo {
        binding_count: 1,
        p_bindings: &ubo_layout_binding,
        ..Default::default()
    };

    let descriptor_set_layout = device.create_descriptor_set_layout(&layout_info, None).unwrap();
    return descriptor_set_layout;
}

unsafe fn create_descriptor_pool(device: &ash::Device) -> vk::DescriptorPool {
    let pool_size = vk::DescriptorPoolSize {
        ty: vk::DescriptorType::UNIFORM_BUFFER,
        descriptor_count: 1,
        ..Default::default()
    };

    let pool_info = vk::DescriptorPoolCreateInfo {
        pool_size_count: 1,
        p_pool_sizes: &pool_size,
        max_sets: 1,
        ..Default::default()
    };

    let descriptor_pool = device.create_descriptor_pool(&pool_info, None).unwrap();
    return descriptor_pool;
}

unsafe fn create_descriptor_sets(device: &ash::Device, pool: vk::DescriptorPool, layout: vk::DescriptorSetLayout, uni_buffer: vk::Buffer, buffer_size: u64) -> Vec<vk::DescriptorSet> {
    let alloc_info = vk::DescriptorSetAllocateInfo {
        descriptor_pool: pool,
        descriptor_set_count: 1,
        p_set_layouts: &layout,
        ..Default::default()
    };

    let descriptor_sets = device.allocate_descriptor_sets(&alloc_info).unwrap();

    let buffer_info = vk::DescriptorBufferInfo {
        buffer: uni_buffer,
        offset: 0,
        range: buffer_size,
        ..Default::default()
    };

    let descriptor_write = vk::WriteDescriptorSet {
        dst_set: descriptor_sets[0],
        dst_binding: 0,
        dst_array_element: 0,
        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
        descriptor_count: 1,
        p_buffer_info: &buffer_info,
        ..Default::default()
    };

    device.update_descriptor_sets(&[descriptor_write], &[]);

    return descriptor_sets;
}

unsafe fn create_buffer(device: &ash::Device, device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
                        size: vk::DeviceSize, usage: vk::BufferUsageFlags, properties: vk::MemoryPropertyFlags) 
                        -> (vk::Buffer, vk::DeviceMemory, vk::MemoryRequirements) {
    let buffer_info = vk::BufferCreateInfo::builder()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let buffer = device.create_buffer(&buffer_info, None).unwrap();
    let memory_req = device.get_buffer_memory_requirements(buffer);

    let memory_index = find_memorytype_index(&memory_req, &device_memory_properties, properties)
        .expect("unable to find suitable memorytype for buffer.");

    let allocation_info = vk::MemoryAllocateInfo {
        allocation_size: memory_req.size,
        memory_type_index: memory_index,
        ..Default::default()
    };

    let memory = device.allocate_memory(&allocation_info, None).unwrap();
    // let mapped_memory_pointer = device.map_memory(memory, 0, memory_req.size, vk::MemoryMapFlags::empty()).unwrap();

    device.bind_buffer_memory(buffer, memory, 0).unwrap();

    return (buffer, memory, memory_req)
}

unsafe fn fill_buffer<T: std::marker::Copy>(device: &ash::Device, buffer_memory: vk::DeviceMemory, buffer_memory_req: &vk::MemoryRequirements, content: &[T]) {
    let pointer = device.map_memory(buffer_memory, 0, buffer_memory_req.size, vk::MemoryMapFlags::empty()).unwrap();
    let mut align = Align::new(pointer, align_of::<T>() as u64, buffer_memory_req.size);
    align.copy_from_slice(&content);
    device.unmap_memory(buffer_memory);
}

unsafe fn get_proj_matrices(cam: &camera::Camera) -> UniformBufferObject {
    let view = Mat4::look_at_rh(cam.position, cam.position + cam.direction, camera::UP);
    let proj = Mat4::perspective_rh(cam.field_of_view, 1920. / 1080., 0.1, 100.);

    let matrices = UniformBufferObject {
        model: Mat4::from_rotation_x(0.),
        view: view,
        proj: proj
    };
    return matrices;
}

fn y_x_to_i(y: u64, x: u64, width: u64) -> u64 {
    return y * width + x;
}

fn main() {
    unsafe {
        let mut base = ExampleBase::new(1920, 1080);
        let renderpass_attachments = [
            vk::AttachmentDescription {
                format: base.surface_format.format,
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

        let subpass = vk::SubpassDescription::builder()
            .color_attachments(&color_attachment_refs)
            .depth_stencil_attachment(&depth_attachment_ref)
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS);

        let renderpass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&renderpass_attachments)
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(&dependencies);

        let renderpass = base.device.create_render_pass(&renderpass_create_info, None).unwrap();

        let framebuffers: Vec<vk::Framebuffer> = base.present_image_views.iter()
            .map(|&present_image_view| {
                let framebuffer_attachments = [present_image_view, base.depth_image_view];
                let frame_buffer_create_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(renderpass)
                    .attachments(&framebuffer_attachments)
                    .width(base.surface_resolution.width)
                    .height(base.surface_resolution.height)
                    .layers(1);

                base.device.create_framebuffer(&frame_buffer_create_info, None).unwrap()
            })
            .collect();

        let mut cam: camera::Camera = Default::default();
        let mut input_state: InputState = Default::default();

        // +++++++++++++++
        const MAP_SIZE: usize = 512;
        // let mut index_buffer_data = std::boxed::Box::new([0u32; 6 * (MAP_SIZE - 1) * (MAP_SIZE - 1)]);
        let mut index_buffer_data: Vec<u32> = Vec::with_capacity(6 * (MAP_SIZE - 1) * (MAP_SIZE - 1));
        index_buffer_data.resize(6 * (MAP_SIZE - 1) * (MAP_SIZE - 1), 0);
        println!("index buffer size: {} integers, {} bytes", index_buffer_data.capacity(), index_buffer_data.capacity() * 4);
        for i in 0 .. MAP_SIZE - 1 {
            for j in 0 .. MAP_SIZE - 1 {
                let tl = y_x_to_i(i as u64, j as u64, MAP_SIZE as u64) as u32;
                let bl = y_x_to_i((i + 1) as u64, j as u64, MAP_SIZE as u64) as u32;
                let tr = y_x_to_i(i as u64, (j + 1) as u64, MAP_SIZE as u64) as u32;
                let br = y_x_to_i((i + 1) as u64, (j + 1) as u64, MAP_SIZE as u64) as u32;

                index_buffer_data[6 * (i * (MAP_SIZE - 1) + j) + 0] = tl;
                index_buffer_data[6 * (i * (MAP_SIZE - 1) + j) + 1] = bl;
                index_buffer_data[6 * (i * (MAP_SIZE - 1) + j) + 2] = tr;
                index_buffer_data[6 * (i * (MAP_SIZE - 1) + j) + 3] = tr;
                index_buffer_data[6 * (i * (MAP_SIZE - 1) + j) + 4] = bl;
                index_buffer_data[6 * (i * (MAP_SIZE - 1) + j) + 5] = br;
            }
        }
        // println!("{:?}", index_buffer_data);

        let (index_buffer, index_buffer_memory, index_buffer_memory_req) = create_buffer(
            &base.device,
            &base.device_memory_properties,
            (index_buffer_data.len() * std::mem::size_of::<u32>()) as u64,
            vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);

        fill_buffer(&base.device, index_buffer_memory, &index_buffer_memory_req, index_buffer_data.as_ref());
        // +++++++++++++++
        let perlin = Perlin::new();
        let mut vertices: Vec<Vertex> = Vec::with_capacity(MAP_SIZE * MAP_SIZE);
        vertices.resize(MAP_SIZE * MAP_SIZE, Vertex{
            pos: [0.0, 0.0, 0.0, 0.0],
            color: [0.0, 1.0, 0.0, 1.0]
        });
        for i in 0 .. MAP_SIZE {
            for j in 0 .. MAP_SIZE {
                let y = 10. * perlin.get([(i as f64) / 100. + 0.1, (j as f64) / 100. + 0.1]);
                // println!("{:?}", y);
                let v = Vertex {
                    pos: [i as f32, y as f32, j as f32, 1.0],
                    color: [0.0, 1.0, 0.0, 1.0],
                };

                vertices[MAP_SIZE * i + j] = v;
            }
        }

        let (vertex_input_buffer, vertex_input_buffer_memory, vertex_buffer_memory_req) = create_buffer(
                &base.device,
                &base.device_memory_properties,
                (vertices.len() * std::mem::size_of::<Vertex>()) as u64, 
                vk::BufferUsageFlags::VERTEX_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);

        fill_buffer(&base.device, vertex_input_buffer_memory, &vertex_buffer_memory_req, &vertices);
        // ++++++++++++++
        let matrices = UniformBufferObject {
            model: Mat4::from_rotation_z(0.5),
            view: Mat4::IDENTITY,
            proj: Mat4::IDENTITY
        };
        let (matrix_buffer, matrix_buffer_memory, matrix_buffer_memory_req) = create_buffer(
            &base.device,
            &base.device_memory_properties,
            std::mem::size_of::<UniformBufferObject>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);
        fill_buffer(&base.device, matrix_buffer_memory, &matrix_buffer_memory_req, &[matrices]);
        // ++++++++++++++

        let mut vertex_spv_file = Cursor::new(&include_bytes!("./shaders/vert.spv")[..]);
        let mut frag_spv_file = Cursor::new(&include_bytes!("./shaders/frag.spv")[..]);
        let vertex_shader_module = get_shader_module(&mut vertex_spv_file, &base.device);
        let fragment_shader_module = get_shader_module(&mut frag_spv_file, &base.device);


        let descriptor_pool = create_descriptor_pool(&base.device);
        let descriptor_set_layout = create_descriptor_set_layout(&base.device);
        let descriptor_sets = create_descriptor_sets(&base.device, descriptor_pool, descriptor_set_layout, matrix_buffer, mem::size_of::<UniformBufferObject>() as u64);
        let layout_create_info = vk::PipelineLayoutCreateInfo {
            set_layout_count: 1,
            p_set_layouts: &descriptor_set_layout,
            ..Default::default()
        };
        let pipeline_layout = base.device.create_pipeline_layout(&layout_create_info, None).unwrap();

        let shader_entry_name = CStr::from_bytes_with_nul_unchecked(b"main\0");
        let shader_stage_create_infos = [
            vk::PipelineShaderStageCreateInfo {
                module: vertex_shader_module,
                p_name: shader_entry_name.as_ptr(),
                stage: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                module: fragment_shader_module,
                p_name: shader_entry_name.as_ptr(),
                stage: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            },
        ];
        let vertex_input_binding_descriptions = [vk::VertexInputBindingDescription {
            binding: 0,
            stride: mem::size_of::<Vertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }];
        let vertex_input_attribute_descriptions = [
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(Vertex, pos) as u32,
            },
            vk::VertexInputAttributeDescription {
                location: 1,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: offset_of!(Vertex, color) as u32,
            },
        ];


        let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&vertex_input_attribute_descriptions)
            .vertex_binding_descriptions(&vertex_input_binding_descriptions);
        let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            ..Default::default()
        };
        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: base.surface_resolution.width as f32,
            height: base.surface_resolution.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];
        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: base.surface_resolution,
        }];
        let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder().scissors(&scissors).viewports(&viewports);

        let rasterization_info = vk::PipelineRasterizationStateCreateInfo {
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            line_width: 1.0,
            polygon_mode: vk::PolygonMode::LINE,
            cull_mode: vk::CullModeFlags::BACK,
            ..Default::default()
        };
        let multisample_state_info = vk::PipelineMultisampleStateCreateInfo {
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            ..Default::default()
        };
        let noop_stencil_state = vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::ALWAYS,
            ..Default::default()
        };
        let depth_state_info = vk::PipelineDepthStencilStateCreateInfo {
            depth_test_enable: 1,
            depth_write_enable: 1,
            depth_compare_op: vk::CompareOp::LESS_OR_EQUAL,
            front: noop_stencil_state,
            back: noop_stencil_state,
            max_depth_bounds: 1.0,
            ..Default::default()
        };
        let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
            blend_enable: 0,
            src_color_blend_factor: vk::BlendFactor::SRC_COLOR,
            dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_DST_COLOR,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ZERO,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
            color_write_mask: vk::ColorComponentFlags::RGBA,
        }];
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op(vk::LogicOp::CLEAR)
            .attachments(&color_blend_attachment_states);

        let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info =
            vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_state);

        let graphic_pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stage_create_infos)
            .vertex_input_state(&vertex_input_state_info)
            .input_assembly_state(&vertex_input_assembly_state_info)
            .viewport_state(&viewport_state_info)
            .rasterization_state(&rasterization_info)
            .multisample_state(&multisample_state_info)
            .depth_stencil_state(&depth_state_info)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state_info)
            .layout(pipeline_layout)
            .render_pass(renderpass);

        let graphics_pipelines = base.device
            .create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[graphic_pipeline_info.build()],
                None,
            )
            .expect("Unable to create graphics pipeline");

        let graphic_pipeline = graphics_pipelines[0];

        let start_time = time::Instant::now();

        let mut mouse_x: f32 = 0.;
        let mut mouse_y: f32 = 0.;

        while !base.window.should_close() {
            base.window.swap_buffers();

            let projection_matrices = get_proj_matrices(&cam);
            fill_buffer(&base.device, matrix_buffer_memory, &matrix_buffer_memory_req, &[projection_matrices]);

            let (present_index, _) = base
                .swapchain_loader
                .acquire_next_image(
                    base.swapchain,
                    std::u64::MAX,
                    base.present_complete_semaphore,
                    vk::Fence::null(),
                )
                .unwrap();
            let clear_values = [
                vk::ClearValue {
                    color: vk::ClearColorValue {
                        float32: [0.0, 0.0, 0.0, 0.0],
                    },
                },
                vk::ClearValue {
                    depth_stencil: vk::ClearDepthStencilValue {
                        depth: 1.0,
                        stencil: 0,
                    },
                },
            ];

            let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
                .render_pass(renderpass)
                .framebuffer(framebuffers[present_index as usize])
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: base.surface_resolution,
                })
                .clear_values(&clear_values);

            record_submit_commandbuffer(
                &base.device,
                base.draw_command_buffer,
                base.draw_commands_reuse_fence,
                base.present_queue,
                &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
                &[base.present_complete_semaphore],
                &[base.rendering_complete_semaphore],
                |device, draw_command_buffer| {
                    device.cmd_begin_render_pass(
                        draw_command_buffer,
                        &render_pass_begin_info,
                        vk::SubpassContents::INLINE,
                    );
                    device.cmd_bind_pipeline(
                        draw_command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        graphic_pipeline,
                    );
                    device.cmd_set_viewport(draw_command_buffer, 0, &viewports);
                    device.cmd_set_scissor(draw_command_buffer, 0, &scissors);
                    device.cmd_bind_vertex_buffers(draw_command_buffer, 0, &[vertex_input_buffer], &[0]);
                    device.cmd_bind_index_buffer(draw_command_buffer, index_buffer, 0, vk::IndexType::UINT32);
                    device.cmd_bind_descriptor_sets(draw_command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline_layout, 0, &descriptor_sets, &[]);
                    device.cmd_draw_indexed(draw_command_buffer, index_buffer_data.len() as u32, 1, 0, 0, 1);
                    // Or draw without the index buffer
                    // device.cmd_draw(draw_command_buffer, 3, 1, 0, 0);
                    device.cmd_end_render_pass(draw_command_buffer);
                },
            );
            //let mut present_info_err = mem::zeroed();
            let wait_semaphors = [base.rendering_complete_semaphore];
            let swapchains = [base.swapchain];
            let image_indices = [present_index];
            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(&wait_semaphors) // &base.rendering_complete_semaphore)
                .swapchains(&swapchains)
                .image_indices(&image_indices);

            base.swapchain_loader.queue_present(base.present_queue, &present_info).unwrap();

            for (_, event) in glfw::flush_messages(&base.events) {
                println!("{:?}", event);
                match event {
                    glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
                        base.window.set_should_close(true)
                    },
                    glfw::WindowEvent::Key(glfw::Key::LeftControl, _, glfw::Action::Press, _) => {
                        input_state.l_ctrl = true;
                    },
                    glfw::WindowEvent::Key(glfw::Key::LeftControl, _, glfw::Action::Release, _) => {
                        input_state.l_ctrl = false;
                    },
                    glfw::WindowEvent::Key(glfw::Key::Space, _, glfw::Action::Press, _) => {
                        input_state.space = true;
                    },
                    glfw::WindowEvent::Key(glfw::Key::Space, _, glfw::Action::Release, _) => {
                        input_state.space = false;
                    },
                    glfw::WindowEvent::Key(glfw::Key::W, _, glfw::Action::Press, _) => {
                        input_state.w = true;
                    },
                    glfw::WindowEvent::Key(glfw::Key::W, _, glfw::Action::Release, _) => {
                        input_state.w = false;
                    },
                    glfw::WindowEvent::Key(glfw::Key::A, _, glfw::Action::Press, _) => {
                        input_state.a = true;
                    },
                    glfw::WindowEvent::Key(glfw::Key::A, _, glfw::Action::Release, _) => {
                        input_state.a = false;
                    },
                    glfw::WindowEvent::Key(glfw::Key::S, _, glfw::Action::Press, _) => {
                        input_state.s = true;
                    },
                    glfw::WindowEvent::Key(glfw::Key::S, _, glfw::Action::Release, _) => {
                        input_state.s = false;
                    },
                    glfw::WindowEvent::Key(glfw::Key::D, _, glfw::Action::Press, _) => {
                        input_state.d = true;
                    },
                    glfw::WindowEvent::Key(glfw::Key::D, _, glfw::Action::Release, _) => {
                        input_state.d = false;
                    },
                    glfw::WindowEvent::CursorPos(x, y) => {
                        let delta_x = mouse_x - x as f32;
                        let delta_y = mouse_y - y as f32;
                        mouse_x = x as f32;
                        mouse_y = y as f32;

                        let new_yaw   = cam.yaw + 0.01 * delta_x;
                        let new_pitch = cam.pitch + 0.01 * delta_y;

                        cam = cam.set_pitch_and_yaw(&new_pitch, &new_yaw);
                    }
                    _ => {},
                }
            }

            if input_state.w {
                cam.position += 0.1 * cam.direction;
            }
            if input_state.a {
                cam.position += 0.1 * cam.direction.cross(camera::UP).normalize();
            }
            if input_state.s {
                cam.position -= 0.1 * cam.direction;
            }
            if input_state.d {
                cam.position -= 0.1 * cam.direction.cross(camera::UP).normalize();
            }
            if input_state.l_ctrl {
                cam.position -= 0.1 * camera::UP;
            }
            if input_state.space {
                cam.position += 0.1 * camera::UP;
            }

            // println!("{:?}", cam.position);
            base.glfw.poll_events();
        }

        base.device.device_wait_idle().unwrap();
        for pipeline in graphics_pipelines {
            base.device.destroy_pipeline(pipeline, None);
        }
        base.device.destroy_descriptor_set_layout(descriptor_set_layout, None);
        base.device.destroy_descriptor_pool(descriptor_pool, None);
        base.device.destroy_pipeline_layout(pipeline_layout, None);
        base.device.destroy_shader_module(vertex_shader_module, None);
        base.device.destroy_shader_module(fragment_shader_module, None);
        base.device.free_memory(index_buffer_memory, None);
        base.device.destroy_buffer(index_buffer, None);
        base.device.free_memory(vertex_input_buffer_memory, None);
        base.device.destroy_buffer(vertex_input_buffer, None);
        base.device.free_memory(matrix_buffer_memory, None);
        base.device.destroy_buffer(matrix_buffer, None);
        for framebuffer in framebuffers {
            base.device.destroy_framebuffer(framebuffer, None);
        }
        base.device.destroy_render_pass(renderpass, None);
    }
}
