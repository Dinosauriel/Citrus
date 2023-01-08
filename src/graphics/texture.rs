use ash::vk;
use image;
use crate::find_memorytype_index;
use crate::graphics::buffer;
use crate::graphics::state::GraphicState;
use super::state::submit_commandbuffer;

pub struct Sampler {
    pub vk_sampler: vk::Sampler
}

impl Sampler {
    unsafe fn create(g_state: &GraphicState) -> Self {
        let sampler_create_info = vk::SamplerCreateInfo {
            mag_filter: vk::Filter::LINEAR,
            min_filter: vk::Filter::LINEAR,
            address_mode_u: vk::SamplerAddressMode::CLAMP_TO_BORDER,
            address_mode_v: vk::SamplerAddressMode::CLAMP_TO_BORDER,
            address_mode_w: vk::SamplerAddressMode::CLAMP_TO_BORDER,
            anisotropy_enable: vk::TRUE,
            max_anisotropy: g_state.instance.get_physical_device_properties(g_state.pdevice).limits.max_sampler_anisotropy,
            border_color: vk::BorderColor::INT_OPAQUE_BLACK,
            unnormalized_coordinates: vk::FALSE,
            compare_enable: vk::FALSE,
            compare_op: vk::CompareOp::ALWAYS,
            mipmap_mode: vk::SamplerMipmapMode::LINEAR,
            mip_lod_bias: 0.,
            min_lod: 0.,
            max_lod: 0.,
            ..Default::default()
        };

        let vk_sampler = g_state.device.create_sampler(&sampler_create_info, None).expect("unable to create sampler");

        Sampler {
            vk_sampler
        }
    }

    unsafe fn free(&self, g_state: &GraphicState) {
        g_state.device.destroy_sampler(self.vk_sampler, None);
    }
}


pub struct ImageView {
    pub vk_image_view: vk::ImageView,
}

impl ImageView {
    unsafe fn create(g_state: &GraphicState, for_image: &Image) -> Self {
        let view_create_info = vk::ImageViewCreateInfo {
            image: for_image.vk_image,
            view_type: vk::ImageViewType::TYPE_2D,
            format: for_image.vk_format,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
                ..Default::default()
            },
            ..Default::default()
        };

        let image_view = g_state.device.create_image_view(&view_create_info, None).expect("unable to creat image view");

        ImageView {
            vk_image_view: image_view
        }
    }

    unsafe fn free(&self, g_state: &GraphicState) {
        g_state.device.destroy_image_view(self.vk_image_view, None);
    }
}

pub struct Image {
    width: u32,
    height: u32,
    vk_image: vk::Image,
    vk_memory: vk::DeviceMemory,
    vk_format: vk::Format,
}

impl Image {
    // create a vulkan image object of size, then bind it
    unsafe fn create(g_state: &GraphicState, width: u32, height: u32, format: vk::Format, tiling: vk::ImageTiling, usage: vk::ImageUsageFlags, properties: vk::MemoryPropertyFlags) -> Self {
        let image_create_info = vk::ImageCreateInfo {
            image_type: vk::ImageType::TYPE_2D,
            extent: vk::Extent3D {
                width, height, depth: 1
            },
            mip_levels: 1,
            array_layers: 1,
            format,
            tiling,
            initial_layout: vk::ImageLayout::UNDEFINED,
            usage,
            samples: vk::SampleCountFlags::TYPE_1,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };

        let vk_image = g_state.device.create_image(&image_create_info, None).expect("cant create vulkan image");

        let memory_requirements = g_state.device.get_image_memory_requirements(vk_image);

        let alloc_info = vk::MemoryAllocateInfo {
            allocation_size: memory_requirements.size,
            memory_type_index: find_memorytype_index(&g_state.device_memory_properties, &memory_requirements, properties).expect("unable to find memory type"),
            ..Default::default()
        };

        let vk_memory = g_state.device.allocate_memory(&alloc_info, None).expect("unable to allocate image memory");

        g_state.device.bind_image_memory(vk_image, vk_memory, 0).expect("unable to bind image memory");

        Image {
            width,
            height,
            vk_image,
            vk_memory,
            vk_format: format
        }
    }

    unsafe fn transition_layout(&self, g_state: &GraphicState, from_layout: vk::ImageLayout, to_layout: vk::ImageLayout) {
        submit_commandbuffer(
            &g_state.device,
            g_state.setup_command_buffer,
            g_state.setup_commands_reuse_fence,
            g_state.present_queue,
            &[], &[], &[],
            | device, command_buffer | {
                let mut barrier = vk::ImageMemoryBarrier {
                    old_layout: from_layout,
                    new_layout: to_layout,
                    src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                    dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                    image: self.vk_image,
                    subresource_range: vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                        ..Default::default()
                    },
                    ..Default::default()
                };

                let source_stage;
                let destination_stage;
                if (from_layout, to_layout) == (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) {
                    // barrier.src_access_mask = 0;
                    barrier.dst_access_mask = vk::AccessFlags::TRANSFER_WRITE;

                    source_stage = vk::PipelineStageFlags::TOP_OF_PIPE;
                    destination_stage = vk::PipelineStageFlags::TRANSFER;
                } else if (from_layout, to_layout) == (vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) {
                    barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
                    barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

                    source_stage = vk::PipelineStageFlags::TRANSFER;
                    destination_stage = vk::PipelineStageFlags::FRAGMENT_SHADER;
                } else {
                    panic!("unsuported layout");
                }

                device.cmd_pipeline_barrier(
                    command_buffer,
                    source_stage,
                    destination_stage,
                    vk::DependencyFlags::BY_REGION, 
                    &[], &[], &[barrier]);
            });
    }

    unsafe fn copy_buffer(&self, g_state: &GraphicState, source_buffer: vk::Buffer) {
        submit_commandbuffer(
            &g_state.device,
            g_state.setup_command_buffer,
            g_state.setup_commands_reuse_fence,
            g_state.present_queue,
            &[], &[], &[],
            | device, command_buffer | {
                let copy_region = vk::BufferImageCopy {
                    buffer_offset: 0,
                    buffer_row_length: 0,
                    buffer_image_height: 0,
                    image_subresource: vk::ImageSubresourceLayers {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        mip_level: 0,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                    image_offset: vk::Offset3D {
                        x: 0, y: 0, z: 0
                    },
                    image_extent: vk::Extent3D {
                        width: self.width,
                        height: self.height,
                        depth: 1
                    },
                    ..Default::default()
                };

                device.cmd_copy_buffer_to_image(
                    command_buffer,
                    source_buffer, 
                    self.vk_image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &[copy_region]);
            });
    }

    pub unsafe fn free(&self, g_state: &GraphicState) {
        g_state.device.destroy_image(self.vk_image, None);
        g_state.device.free_memory(self.vk_memory, None);
    }
}

pub struct Texture {
    image: Image,
    pub image_view: ImageView,
    pub sampler: Sampler,
}

impl Texture {
    pub unsafe fn create_from_bytes(g_state: &GraphicState, bytes: &[u8], width: u32, height: u32) -> Self {
        let img: image::ImageBuffer<image::Rgba<u8>, &[u8]> = image::ImageBuffer::from_raw(width, height, bytes).expect("couldnt read raw image");
        img.save("texture.png").expect("could not save image");

        let buffer_size: vk::DeviceSize = 4 * (width as u64) * (height as u64);

        let buffer = buffer::Buffer::create(
            &g_state.device,
            &g_state.device_memory_properties,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE |  vk::MemoryPropertyFlags::HOST_COHERENT);
        buffer.fill(&g_state.device, bytes);

        let image = Image::create(
            g_state,
            width,
            height,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            vk::MemoryPropertyFlags::DEVICE_LOCAL);

        image.transition_layout(g_state, vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL);
        image.copy_buffer(g_state, buffer.vk_buffer);
        image.transition_layout(g_state, vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);

        buffer.free(&g_state.device);

        let image_view = ImageView::create(g_state, &image);
        let sampler = Sampler::create(g_state);

        Texture {
            image,
            image_view,
            sampler
        }
    }

    pub unsafe fn free(&self, g_state: &GraphicState) {
        self.image.free(g_state);
        self.image_view.free(g_state);
        self.sampler.free(g_state);
    }
}
