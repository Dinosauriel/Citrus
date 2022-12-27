use ash::vk;
use glam::Mat4;
use citrus::*;
use std::default::Default;
use std::ffi::CStr;
use std::io::Cursor;
use std::mem;
use std::time;
use glfw::Context;
use controls::InputState;
use world::*;
use world::size::*;
use world::object::*;
use graphics::shader::*;
use graphics::object::*;
use graphics::buffer;

#[derive(Clone, Debug, Copy)]
#[allow(dead_code)]
struct UniformBufferObject {
    model: Mat4,
    view: Mat4,
    proj: Mat4,
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

unsafe fn get_proj_matrices(cam: &camera::Camera) -> UniformBufferObject {
    let view = Mat4::look_at_rh(cam.ray.origin, cam.ray.origin + cam.ray.direction, camera::UP);
    // let view = Mat4::look_at_rh(- cam.direction * 2., glam::Vec3::new(0., 0., 0.), camera::UP);
    let proj = Mat4::perspective_rh(cam.field_of_view, 1920. / 1080., 0.1, 100.);

    let matrices = UniformBufferObject {
        model: Mat4::IDENTITY,
        view: view,
        proj: proj
    };
    return matrices;
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

        let mut cam = camera::Camera::default();
        let mut input_state = InputState::default();

        // +++++++++++++++
        println!("worldgen");


        let mut world = World::new();

        let mut object1 = BlockObject::new(Size3D {x: 2, y: 2, z: 2}, glam::Vec3::new(10., 10., 0.), vec![BlockType::Grass; 8]);
        object1.update_indices();
        object1.update_vertices();
        world.objects.push(object1);

        let mut object_buffers: Vec<(&BlockObject, buffer::Buffer, buffer::Buffer)> = Vec::with_capacity(world.objects.len());
        
        for object in &world.objects {

            let index_buffer = buffer::Buffer::create(
                &base.device,
                &base.device_memory_properties,
                (object.indices().len() * std::mem::size_of::<u32>()) as u64,
                vk::BufferUsageFlags::INDEX_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);

            index_buffer.fill(&base.device, &object.indices());

            let vertex_buffer = buffer::Buffer::create(
                &base.device,
                &base.device_memory_properties,
                (object.vertices().len() * std::mem::size_of::<Vertex>()) as u64, 
                vk::BufferUsageFlags::VERTEX_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);

            vertex_buffer.fill(&base.device, &object.vertices());

            object_buffers.push((object, vertex_buffer, index_buffer));
        }

        let n = world.objects.len();
        println!("{n} objects found");

        let triangle = Triangle::new(
            &Vertex {
                pos: [0., 2., 0., 1.],
                color: [1., 0., 1., 1.]
            },
            &Vertex {
                pos: [0., -2., 0., 1.],
                color: [1., 0., 0., 1.]
            },
            &Vertex {
                pos: [1., 0., 0., 0.],
                color: [1., 0., 0., 1.]
            },
        );

        let triangle_index_buffer = buffer::Buffer::create(
            &base.device,
            &base.device_memory_properties,
            (triangle.indices().len() * std::mem::size_of::<u32>()) as u64,
            vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);
        triangle_index_buffer.fill(&base.device, triangle.indices());

        let triangle_vertex_buffer = buffer::Buffer::create(
            &base.device,
            &base.device_memory_properties,
            (triangle.vertices().len() * std::mem::size_of::<Vertex>()) as u64, 
            vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);
        triangle_vertex_buffer.fill(&base.device, triangle.vertices());
        // ++++++++++++++

        let hud_triangle = Triangle::new(
            &Vertex {
                pos: [0.5, 0., 0., 1.],
                color: [1., 0., 1., 1.]
            },
            &Vertex {
                pos: [-0.5, 0., 0., 1.],
                color: [1., 0., 0., 1.]
            },
            &Vertex {
                pos: [0., 0.5, 0., 1.],
                color: [1., 1., 0., 1.]
            },
        );
        let hud_triangle_index_buffer = buffer::Buffer::create(
            &base.device,
            &base.device_memory_properties,
            (hud_triangle.indices().len() * std::mem::size_of::<u32>()) as u64,
            vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);
        hud_triangle_index_buffer.fill(&base.device, hud_triangle.indices());

        let hud_triangle_vertex_buffer = buffer::Buffer::create(
            &base.device,
            &base.device_memory_properties,
            (hud_triangle.vertices().len() * std::mem::size_of::<Vertex>()) as u64, 
            vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);
        hud_triangle_vertex_buffer.fill(&base.device, hud_triangle.vertices());

        // ++++++++++++++
        let matrix_buffer = buffer::Buffer::create(
            &base.device,
            &base.device_memory_properties,
            std::mem::size_of::<UniformBufferObject>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);
        // ++++++++++++++

        let descriptor_pool = create_descriptor_pool(&base.device);
        let descriptor_set_layout = create_descriptor_set_layout(&base.device);
        let descriptor_sets = create_descriptor_sets(&base.device, descriptor_pool, descriptor_set_layout, matrix_buffer.vulkan_instance, mem::size_of::<UniformBufferObject>() as u64);
        let layout_create_info = vk::PipelineLayoutCreateInfo {
            set_layout_count: 1,
            p_set_layouts: &descriptor_set_layout,
            ..Default::default()
        };
        let pipeline_layout = base.device.create_pipeline_layout(&layout_create_info, None).unwrap();

        let mut vertex_spv_file = Cursor::new(&include_bytes!("./shaders/vert.spv")[..]);
        let mut frag_spv_file = Cursor::new(&include_bytes!("./shaders/frag.spv")[..]);
        let vertex_shader_module = get_shader_module(&mut vertex_spv_file, &base.device);
        let fragment_shader_module = get_shader_module(&mut frag_spv_file, &base.device);

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

        let mut hud_vertex_spv = Cursor::new(&include_bytes!("./shaders/hud_vert.spv")[..]);
        let mut hud_frag_spv = Cursor::new(&include_bytes!("./shaders/hud_frag.spv")[..]);
        let hud_vertex_shader_module = get_shader_module(&mut hud_vertex_spv, &base.device);
        let hud_fragment_shader_module = get_shader_module(&mut hud_frag_spv, &base.device);

        let hud_shader_stage_create_infos = [
            vk::PipelineShaderStageCreateInfo {
                module: hud_vertex_shader_module,
                p_name: shader_entry_name.as_ptr(),
                stage: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                module: hud_fragment_shader_module,
                p_name: shader_entry_name.as_ptr(),
                stage: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            },
        ];


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
        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_state);

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

        let hud_pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&hud_shader_stage_create_infos)
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
                &[graphic_pipeline_info.build(), hud_pipeline_info.build()],
                None,
            )
            .expect("Unable to create graphics pipeline");

        let mut last_second = time::Instant::now();
        let mut frames = 0;
        let mut ticks = 0;

        let mut last_tick = time::Instant::now();
        const SECONDS_PER_TICK: f64 = (1 as f64) / (config::TICK_RATE as f64);
        println!("target seconds per tick: {:?}", time::Duration::from_secs_f64(SECONDS_PER_TICK));

        while !base.window.should_close() {

            if last_second.elapsed() >= time::Duration::from_secs(1) {
                println!("FPS: {}, TPS: {}", frames, ticks);
                last_second = time::Instant::now();
                frames = 0;
                ticks = 0;

                print!("camera intersects:");
                for c in cam.ray.intersected_blocks(5) {
                    print!("({}, {}, {}) ", c.x, c.y, c.z);
                }
                println!();
            }

            base.window.swap_buffers();

            let projection_matrices = get_proj_matrices(&cam);
            matrix_buffer.fill(&base.device, &[projection_matrices]);

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
                | device: &ash::Device, draw_command_buffer: vk::CommandBuffer | {
                    device.cmd_begin_render_pass(draw_command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
                    device.cmd_bind_pipeline(draw_command_buffer, vk::PipelineBindPoint::GRAPHICS, graphics_pipelines[0]);

                    device.cmd_set_viewport(draw_command_buffer, 0, &viewports);
                    device.cmd_set_scissor(draw_command_buffer, 0, &scissors);
                    device.cmd_bind_descriptor_sets(draw_command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline_layout, 0, &descriptor_sets, &[]);
                    
                    device.cmd_bind_vertex_buffers(draw_command_buffer, 0, &[triangle_vertex_buffer.vulkan_instance], &[0]);
                    device.cmd_bind_index_buffer(draw_command_buffer, triangle_index_buffer.vulkan_instance, 0, vk::IndexType::UINT32);
                    device.cmd_draw_indexed(draw_command_buffer, triangle.indices().len() as u32, 1, 0, 0, 1);

                    for (object, vertex_buffer, index_buffer) in &object_buffers {
                        device.cmd_bind_vertex_buffers(draw_command_buffer, 0, &[vertex_buffer.vulkan_instance], &[0]);
                        device.cmd_bind_index_buffer(draw_command_buffer, index_buffer.vulkan_instance, 0, vk::IndexType::UINT32);
                        device.cmd_draw_indexed(draw_command_buffer, object.indices().len() as u32, 1, 0, 0, 1);
                    }

                    device.cmd_bind_pipeline(draw_command_buffer, vk::PipelineBindPoint::GRAPHICS, graphics_pipelines[1]);
                    device.cmd_bind_vertex_buffers(draw_command_buffer, 0, &[hud_triangle_vertex_buffer.vulkan_instance], &[0]);
                    device.cmd_bind_index_buffer(draw_command_buffer, hud_triangle_index_buffer.vulkan_instance, 0, vk::IndexType::UINT32);
                    device.cmd_draw_indexed(draw_command_buffer, hud_triangle.indices().len() as u32, 1, 0, 0, 1);

                    device.cmd_end_render_pass(draw_command_buffer);
                },
            );

            let wait_semaphors = [base.rendering_complete_semaphore];
            let swapchains = [base.swapchain];
            let image_indices = [present_index];
            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(&wait_semaphors)
                .swapchains(&swapchains)
                .image_indices(&image_indices);

            base.swapchain_loader.queue_present(base.present_queue, &present_info).unwrap();

            if last_tick.elapsed() >= time::Duration::from_secs_f64(SECONDS_PER_TICK) {
                ticks += 1;
                last_tick += time::Duration::from_secs_f64(SECONDS_PER_TICK);
                for (_, event) in glfw::flush_messages(&base.events) {
                    // println!("{:?}", event);
                    input_state.update_from_event(&event);
                }
                cam.update_from_input_state(&input_state);
    
                if input_state.escape {
                    base.window.set_should_close(true);
                    println!("escape!");
                }
            }

            // println!("{:?}", cam.position);
            base.glfw.poll_events();
            frames += 1;
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
        base.device.destroy_shader_module(hud_vertex_shader_module, None);
        base.device.destroy_shader_module(hud_fragment_shader_module, None);

        triangle_index_buffer.free(&base.device);
        hud_triangle_index_buffer.free(&base.device);

        triangle_vertex_buffer.free(&base.device);
        hud_triangle_vertex_buffer.free(&base.device);
        
        for (_, vertex_buffer, index_buffer) in &object_buffers {
            vertex_buffer.free(&base.device);
            index_buffer.free(&base.device);
        }

        matrix_buffer.free(&base.device);

        for framebuffer in framebuffers {
            base.device.destroy_framebuffer(framebuffer, None);
        }
        base.device.destroy_render_pass(renderpass, None);
        println!("free!");
    }
}
