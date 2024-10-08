use std::default::Default;
use std::mem;
use std::time;
use std::time::Duration;
use citrus::random;
use glam::Mat4;
use glam::Vec2;
use ash::vk;
use citrus::{
    profiler::*,
    config,
    ui,
    controls::*,
    world::{
        *,
        block::*,
    },
    graphics::{
        shader::*,
        graphics_object::*,
        vertex::*,
        buffer::*,
        camera::*,
        graphics_state::*,
        texture::*,
        geometry::*,
        pipeline::*,
    }
};

#[repr(C)]
#[derive(Clone, Debug, Copy)]
#[allow(dead_code)]
struct WorldUBO {
    model: Mat4,
    view: Mat4,
    proj: Mat4,
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
#[allow(dead_code)]
struct HudUBO {
    scale: Mat4,
}

unsafe fn create_descriptor_set_layout(device: &ash::Device) -> ash::vk::DescriptorSetLayout {
    let ubo_layout_binding = vk::DescriptorSetLayoutBinding {
        binding: 0,
        descriptor_count: 1,
        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
        stage_flags: vk::ShaderStageFlags::VERTEX,
        ..Default::default()
    };

    let hud_ubo_layout_binding = vk::DescriptorSetLayoutBinding {
        binding: 2,
        descriptor_count: 1,
        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
        stage_flags: vk::ShaderStageFlags::VERTEX,
        ..Default::default()
    };

    let sampler_layout_binding = vk::DescriptorSetLayoutBinding {
        binding: 1,
        descriptor_count: 1,
        descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        stage_flags: vk::ShaderStageFlags::FRAGMENT,
        ..Default::default()
    };

    let bindings = [ubo_layout_binding, hud_ubo_layout_binding, sampler_layout_binding];

    let layout_info = vk::DescriptorSetLayoutCreateInfo {
        binding_count: bindings.len() as u32,
        p_bindings: bindings.as_ptr(),
        ..Default::default()
    };

    device.create_descriptor_set_layout(&layout_info, None).unwrap()
}

unsafe fn create_descriptor_pool(device: &ash::Device) -> vk::DescriptorPool {
    let uniform_pool_size = vk::DescriptorPoolSize {
        ty: vk::DescriptorType::UNIFORM_BUFFER,
        descriptor_count: 2,
        ..Default::default()
    };

    let sampler_pool_size = vk::DescriptorPoolSize {
        ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        descriptor_count: 1,
        ..Default::default()
    };

    let pool_sizes = [uniform_pool_size, sampler_pool_size];

    let pool_info = vk::DescriptorPoolCreateInfo {
        pool_size_count: pool_sizes.len() as u32,
        p_pool_sizes: pool_sizes.as_ptr(),
        max_sets: 1,
        ..Default::default()
    };

    let descriptor_pool = device.create_descriptor_pool(&pool_info, None).unwrap();
    descriptor_pool
}

// allocate a new descriptor set for a texture sampler and a uniform buffer object
unsafe fn create_descriptor_sets(device: &ash::Device, pool: vk::DescriptorPool, layout: vk::DescriptorSetLayout, 
                            uni_buffer: vk::Buffer, hud_uni_buffer: vk::Buffer, texture: &Texture) -> Vec<vk::DescriptorSet> {
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
        range: mem::size_of::<WorldUBO>() as u64,
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

    let hud_buffer_info = vk::DescriptorBufferInfo {
        buffer: hud_uni_buffer,
        offset: 0,
        range: mem::size_of::<HudUBO>() as u64,
        ..Default::default()
    };

    let hud_descriptor_write = vk::WriteDescriptorSet {
        dst_set: descriptor_sets[0],
        dst_binding: 2,
        dst_array_element: 0,
        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
        descriptor_count: 1,
        p_buffer_info: &hud_buffer_info,
        ..Default::default()
    };

    let image_info = vk::DescriptorImageInfo {
        image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        image_view: texture.image_view.vk_image_view,
        sampler: texture.sampler.vk_sampler,
        ..Default::default()
    };

    let sampler_descriptor_write = vk::WriteDescriptorSet {
        dst_set: descriptor_sets[0],
        dst_binding: 1,
        dst_array_element: 0,
        descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        descriptor_count: 1,
        p_image_info: &image_info,
        ..Default::default()
    };

    device.update_descriptor_sets(&[descriptor_write, hud_descriptor_write, sampler_descriptor_write], &[]);
    descriptor_sets
}

fn get_world_ubo(cam: &Camera) -> WorldUBO {
    WorldUBO {
        model: Mat4::IDENTITY,
        view: Mat4::look_at_rh(cam.ray.origin, cam.ray.origin + cam.ray.direction, UP),
        proj: Mat4::perspective_rh(cam.field_of_view, 1920. / 1080., 0.1, 1000.),
    }
}

fn get_hud_ubo() -> HudUBO {
    HudUBO {
        scale: Mat4::from_cols(
            glam::Vec4::new(1. / 960., 0.0, 0.0, 0.0),
            glam::Vec4::new(0.0, 1. / 540., 0.0, 0.0),
            glam::Vec4::new(0.0, 0.0, 1., 0.0),
            glam::Vec4::new(0.0, 0.0, 0.0, 1.0),
        ),
    }
}

fn main() {
    unsafe {
        let mut g_state = GraphicState::new(1920, 1080);

        let mut mt = random::mt::Mt19937::new(123);
        for _ in 0..14 {
            println!("{}", mt.next());
        }

        let render_pass = render_pass(&g_state);

        let framebuffers: Vec<vk::Framebuffer> = g_state.present_image_views.iter()
            .map(|&present_image_view| {
                let framebuffer_attachments = [present_image_view, g_state.depth_image_view];
                let frame_buffer_create_info = vk::FramebufferCreateInfo::default()
                    .render_pass(render_pass)
                    .attachments(&framebuffer_attachments)
                    .width(g_state.surface_resolution.width)
                    .height(g_state.surface_resolution.height)
                    .layers(1);

                g_state.device.create_framebuffer(&frame_buffer_create_info, None).unwrap()
            })
            .collect();

        let mut cam = Camera::default();
        let mut input_state = InputState::default();

        let deja_vu = ui::font::Font::load(&g_state, "./src/assets/DejaVuSansMono.ttf", 48);

        let mut dummy_text = ui::text::Text::new(&g_state.device, &g_state.device_memory_properties, 32);

        let world = World::new(&g_state.device, &g_state.device_memory_properties);

        let mut blocks = vec![BlockType::Grass; 8];
        blocks[0] = BlockType::NoBlock;

        // ++++++++++++++
        let matrix_buffer = Buffer::new(
            &g_state.device,
            &g_state.device_memory_properties,
            std::mem::size_of::<WorldUBO>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);
        // ++++++++++++++
        let hud_matrix_buffer = Buffer::new(
            &g_state.device,
            &g_state.device_memory_properties,
            std::mem::size_of::<HudUBO>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);
        // ++++++++++++++
        hud_matrix_buffer.fill(&[get_hud_ubo()]);

        let descriptor_pool = create_descriptor_pool(&g_state.device);
        let descriptor_set_layout = create_descriptor_set_layout(&g_state.device);
        let descriptor_sets = create_descriptor_sets(
            &g_state.device, descriptor_pool,
            descriptor_set_layout, matrix_buffer.vk_buffer, 
            hud_matrix_buffer.vk_buffer, &deja_vu.texture);

        let pipeline_layout = pipeline_layout(&g_state, &descriptor_set_layout);

        let world_fragment_shader_module = shader_module(&g_state, ShaderType::World, ShaderStage::Fragment);
        let world_vertex_shader_module = shader_module(&g_state, ShaderType::World, ShaderStage::Vertex);
        let hud_fragment_shader_module = shader_module(&g_state, ShaderType::Hud, ShaderStage::Fragment);
        let hud_vertex_shader_module = shader_module(&g_state, ShaderType::Hud, ShaderStage::Vertex);

        let world_shader_stages = shader_stage_create_infos(world_vertex_shader_module, world_fragment_shader_module);
        let hud_shader_stages = shader_stage_create_infos(hud_vertex_shader_module, hud_fragment_shader_module);

        let viewports = viewports(&g_state);
        let scissors = scissors(&g_state);

        let colored_attrs = ColoredVertex::attribute_desctiptions();
        let colored_bindings = ColoredVertex::binding_description();
        let colored_input_state = vertex_input_state(&colored_bindings, &colored_attrs);

        let textured_attrs = TexturedVertex::attribute_desctiptions();
        let textured_bindings = TexturedVertex::binding_description();
        let textured_input_state = vertex_input_state(&textured_bindings, &textured_attrs);

        let world_pipeline = Pipeline::new(PipelineType::World, &scissors, &viewports);
        let graphic_pipeline_info = world_pipeline.create_info(&world_shader_stages, &colored_input_state, render_pass, pipeline_layout);
        
        let world_line_pipeline = Pipeline::new(PipelineType::WorldLine, &scissors, &viewports);
        let line_pipeline_info = world_line_pipeline.create_info(&world_shader_stages, &colored_input_state, render_pass, pipeline_layout);

        let hud_pipeline = Pipeline::new(PipelineType::Hud, &scissors, &viewports);
        let hud_pipeline_info = hud_pipeline.create_info(&hud_shader_stages, &textured_input_state, render_pass, pipeline_layout);

        let graphics_pipelines = g_state.device
            .create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[graphic_pipeline_info, hud_pipeline_info, line_pipeline_info],
                None,
            )
            .expect("Unable to create graphics pipeline");

        let mut last_second = time::Instant::now();
        let mut frames = 0;
        let mut ticks = 0;

        let mut last_tick = time::Instant::now();
        const SECONDS_PER_TICK: f64 = (1 as f64) / (config::TICK_RATE as f64);
        println!("target seconds per tick: {:?}", time::Duration::from_secs_f64(SECONDS_PER_TICK));

        let channel = ui::io::command_line::stdin_channel();

        let mut pipeline_index: usize = 0;

        while !g_state.window.should_close() {

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
            
            matrix_buffer.fill(&[get_world_ubo(&cam)]);

            dummy_text.update(&format!("{:.2} {:.2} {:.2}", cam.ray.origin.x, cam.ray.origin.y, cam.ray.origin.z), &deja_vu, &Vec2::new(860., 440.), (XDir::XPos, YDir::YPos));

            let (present_index, _) = g_state
                .swapchain_loader
                .acquire_next_image(
                    g_state.swapchain,
                    std::u64::MAX,
                    g_state.present_complete_semaphore,
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

            let render_pass_begin_info = vk::RenderPassBeginInfo::default()
                .render_pass(render_pass)
                .framebuffer(framebuffers[present_index as usize])
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: g_state.surface_resolution,
                })
                .clear_values(&clear_values);

            submit_commandbuffer(
                &g_state.device,
                g_state.draw_command_buffer,
                g_state.draw_commands_reuse_fence,
                g_state.present_queue,
                &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
                &[g_state.present_complete_semaphore],
                &[g_state.rendering_complete_semaphore],
                | device: &ash::Device, draw_command_buffer: vk::CommandBuffer | {
                    device.cmd_begin_render_pass(draw_command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
                    device.cmd_bind_pipeline(draw_command_buffer, vk::PipelineBindPoint::GRAPHICS, graphics_pipelines[pipeline_index]);

                    device.cmd_set_viewport(draw_command_buffer, 0, &viewports);
                    device.cmd_set_scissor(draw_command_buffer, 0, &scissors);
                    device.cmd_bind_descriptor_sets(draw_command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline_layout, 0, &descriptor_sets, &[]);
                    
                    // device.cmd_bind_vertex_buffers(draw_command_buffer, 0, &[triangle.vertex_buffer().vk_buffer], &[0]);
                    // device.cmd_bind_index_buffer(draw_command_buffer, triangle.index_buffer().vk_buffer, 0, vk::IndexType::UINT32);
                    // device.cmd_draw_indexed(draw_command_buffer, triangle.indices().len() as u32, 1, 0, 0, 1);

                    for object in &world.objects {
                        device.cmd_bind_vertex_buffers(draw_command_buffer, 0, &[object.vertex_buffer.vk_buffer], &[0]);
                        device.cmd_bind_index_buffer(draw_command_buffer, object.index_buffer.vk_buffer, 0, vk::IndexType::UINT32);
                        device.cmd_draw_indexed(draw_command_buffer, object.index_count, 1, 0, 0, 1);
                    }

                    device.cmd_bind_pipeline(draw_command_buffer, vk::PipelineBindPoint::GRAPHICS, graphics_pipelines[1]);

                    device.cmd_bind_vertex_buffers(draw_command_buffer, 0, &[dummy_text.vertex_buffer().vk_buffer], &[0]);
                    device.cmd_bind_index_buffer(draw_command_buffer, dummy_text.index_buffer().vk_buffer, 0, vk::IndexType::UINT32);
                    device.cmd_draw_indexed(draw_command_buffer, 6 * dummy_text.len() as u32, 1, 0, 0, 1);

                    device.cmd_end_render_pass(draw_command_buffer);
                },
            );

            let wait_semaphores = [g_state.rendering_complete_semaphore];
            let swapchains = [g_state.swapchain];
            let image_indices = [present_index];
            let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(&wait_semaphores)
                .swapchains(&swapchains)
                .image_indices(&image_indices);

            g_state.swapchain_loader.queue_present(g_state.present_queue, &present_info).unwrap();

            if last_tick.elapsed() >= time::Duration::from_secs_f64(SECONDS_PER_TICK) {
                ticks += 1;
                last_tick += time::Duration::from_secs_f64(SECONDS_PER_TICK);
                for (_, event) in glfw::flush_messages(&g_state.events) {
                    // println!("{:?}", event);
                    input_state.update_from_event(&event);
                }
                cam.update_from_input_state(&input_state);
    
                if input_state.escape {
                    g_state.window.set_should_close(true);
                    println!("escape!");
                }

                if input_state.m {
                    pipeline_index = 2;
                } else {
                    pipeline_index = 0;
                }

                // for (object, vertex_buffer, _) in &mut object_buffers {
                //     if object.is_ticking {
                //         let now = time::SystemTime::now().duration_since(time::SystemTime::UNIX_EPOCH).expect("time went backwards");
                //         object.tick(now.as_millis());
                //         object.update_vertices();
                //         vertex_buffer.fill(object.vertices());
                //     }
                // }

                let cmds = ui::io::command_line::commands(&channel);
                if cmds.len() > 0 {
                    println!("{:?}", cmds);
                }
            }

            // println!("{:?}", cam.position);
            g_state.glfw.poll_events();
            frames += 1;
        }

        g_state.device.device_wait_idle().unwrap();
        for pipeline in graphics_pipelines {
            g_state.device.destroy_pipeline(pipeline, None);
        }
        g_state.device.destroy_descriptor_set_layout(descriptor_set_layout, None);
        g_state.device.destroy_descriptor_pool(descriptor_pool, None);

        g_state.device.destroy_pipeline_layout(pipeline_layout, None);

        g_state.device.destroy_shader_module(world_vertex_shader_module, None);
        g_state.device.destroy_shader_module(world_fragment_shader_module, None);
        g_state.device.destroy_shader_module(hud_vertex_shader_module, None);
        g_state.device.destroy_shader_module(hud_fragment_shader_module, None);

        // triangle.index_buffer().free(&g_state.device);
        // triangle.vertex_buffer().free(&g_state.device);

        deja_vu.texture.free(&g_state);
        dummy_text.index_buffer().free(&g_state.device);
        dummy_text.vertex_buffer().free(&g_state.device);

        for object in &world.objects {
            object.vertex_buffer.free(&g_state.device);
            object.index_buffer.free(&g_state.device);
        }

        matrix_buffer.free(&g_state.device);
        hud_matrix_buffer.free(&g_state.device);

        for framebuffer in framebuffers {
            g_state.device.destroy_framebuffer(framebuffer, None);
        }
        g_state.device.destroy_render_pass(render_pass, None);
    }
    p_summary();
    p_graph(Duration::from_millis(10));
}
