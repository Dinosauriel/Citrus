use std::ops::Drop;
use std::ffi::CStr;
use std::os::raw::c_char;
use glfw::Context;
use ash;
use ash::vk;
use ash::extensions::{
    ext::DebugUtils,
    khr::{Surface, Swapchain},
};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use crate::find_memorytype_index;
use crate::vulkan_debug_callback;

/// Helper function for submitting command buffers. Immediately waits for the fence before the command buffer
/// is executed. That way we can delay the waiting for the fences by 1 frame which is good for performance.
/// Make sure to create the fence in a signaled state on the first use.
#[allow(clippy::too_many_arguments)]
pub unsafe fn submit_commandbuffer<F: FnOnce(&ash::Device, vk::CommandBuffer)>(
    device: &ash::Device,
    command_buffer: vk::CommandBuffer,
    command_buffer_reuse_fence: vk::Fence,
    submit_queue: vk::Queue,
    wait_mask: &[vk::PipelineStageFlags],
    wait_semaphores: &[vk::Semaphore],
    signal_semaphores: &[vk::Semaphore],
    f: F,
) {
    device.wait_for_fences(&[command_buffer_reuse_fence], true, std::u64::MAX).expect("Wait for fence failed.");

    device.reset_fences(&[command_buffer_reuse_fence]).expect("Reset fences failed.");

    device.reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::RELEASE_RESOURCES,).expect("Reset command buffer failed.");

    let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    device.begin_command_buffer(command_buffer, &command_buffer_begin_info).expect("unable to begin commandbuffer");

    f(device, command_buffer);

    device.end_command_buffer(command_buffer).expect("unable to end commandbuffer");

    let command_buffers = vec![command_buffer];

    let submit_info = vk::SubmitInfo::builder()
        .wait_semaphores(wait_semaphores)
        .wait_dst_stage_mask(wait_mask)
        .command_buffers(&command_buffers)
        .signal_semaphores(signal_semaphores);

    device.queue_submit(submit_queue, &[submit_info.build()], command_buffer_reuse_fence).expect("queue submit failed.");
}

unsafe fn create_instance(window: &glfw::Window, entry: &ash::Entry) -> ash::Instance {
    let app_name = CStr::from_bytes_with_nul_unchecked(b"VulkanTriangle\0");

    let layer_names = [CStr::from_bytes_with_nul_unchecked(
        b"VK_LAYER_KHRONOS_validation\0",
    )];
    let layers_names_raw: Vec<*const c_char> = layer_names
        .iter()
        .map(|raw_name| raw_name.as_ptr())
        .collect();

    let mut surface_extensions = ash_window::enumerate_required_extensions(window.raw_display_handle()).unwrap().to_vec();
    surface_extensions.push(DebugUtils::name().as_ptr());

    let appinfo = vk::ApplicationInfo::builder()
        .application_name(app_name)
        .application_version(0)
        .engine_name(app_name)
        .engine_version(0)
        .api_version(vk::make_api_version(0, 1, 0, 0));

    let create_info = vk::InstanceCreateInfo::builder()
        .application_info(&appinfo)
        .enabled_layer_names(&layers_names_raw)
        .enabled_extension_names(&surface_extensions);

    let instance: ash::Instance = entry
        .create_instance(&create_info, None)
        .expect("Instance creation error");
    return instance;
}

fn create_debug_callback(entry: &ash::Entry, instance: &ash::Instance) -> (DebugUtils, vk::DebugUtilsMessengerEXT) {
    let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
    .message_severity(
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
            | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
            | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
    )
    .message_type(
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
            | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
    )
    .pfn_user_callback(Some(vulkan_debug_callback));

    let debug_utils_loader = DebugUtils::new(entry, instance);
    unsafe {
        let debug_call_back = debug_utils_loader
            .create_debug_utils_messenger(&debug_info, None)
            .unwrap();
        return (debug_utils_loader, debug_call_back);
    }
}

unsafe fn create_device(instance: &ash::Instance, pdevice: vk::PhysicalDevice, queue_family_index: u32) -> ash::Device {
    let device_extension_names_raw = [Swapchain::name().as_ptr()];

    //the features that we request from the device
    let features = vk::PhysicalDeviceFeatures {
        shader_clip_distance: 1,
        fill_mode_non_solid: 1,
        sampler_anisotropy: vk::TRUE,
        ..Default::default()
    };
    let priorities = [1.0];

    let queue_info = vk::DeviceQueueCreateInfo::builder()
        .queue_family_index(queue_family_index)
        .queue_priorities(&priorities);

    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(std::slice::from_ref(&queue_info))
        .enabled_extension_names(&device_extension_names_raw)
        .enabled_features(&features);

    let device: ash::Device = instance
        .create_device(pdevice, &device_create_info, None)
        .unwrap();

    return device;
}

unsafe fn create_command_buffers(device: &ash::Device, queue_family: u32) -> (vk::CommandPool, vk::CommandBuffer, vk::CommandBuffer) {
    let pool_create_info = vk::CommandPoolCreateInfo::builder()
    .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
    .queue_family_index(queue_family);

    let pool = device.create_command_pool(&pool_create_info, None).unwrap();

    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_buffer_count(2)
        .command_pool(pool)
        .level(vk::CommandBufferLevel::PRIMARY);

    let command_buffers = device
        .allocate_command_buffers(&command_buffer_allocate_info)
        .unwrap();
    let setup_command_buffer = command_buffers[0];
    let draw_command_buffer = command_buffers[1];

    (pool, setup_command_buffer, draw_command_buffer)
}

unsafe fn create_swapchain(instance: &ash::Instance, device: &ash::Device, pdevice: &vk::PhysicalDevice, 
        surface: vk::SurfaceKHR, surface_capabilities: &vk::SurfaceCapabilitiesKHR, surface_loader: &Surface, surface_format: vk::SurfaceFormatKHR,
        surface_resolution: vk::Extent2D) -> (vk::SwapchainKHR, Swapchain) {

    let mut desired_image_count = surface_capabilities.min_image_count + 1;
    if surface_capabilities.max_image_count > 0 {
        desired_image_count = desired_image_count.min(surface_capabilities.max_image_count);
    }
    
    let pre_transform = if surface_capabilities
        .supported_transforms
        .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
    {
        vk::SurfaceTransformFlagsKHR::IDENTITY
    } else {
        surface_capabilities.current_transform
    };
    let present_modes = surface_loader
        .get_physical_device_surface_present_modes(*pdevice, surface)
        .unwrap();
    let present_mode = present_modes
        .iter()
        .cloned()
        .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
        .unwrap_or(vk::PresentModeKHR::FIFO);
    let swapchain_loader = Swapchain::new(&instance, &device);

    let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
        .surface(surface)
        .min_image_count(desired_image_count)
        .image_color_space(surface_format.color_space)
        .image_format(surface_format.format)
        .image_extent(surface_resolution)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
        .pre_transform(pre_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(present_mode)
        .clipped(true)
        .image_array_layers(1);

    let swapchain = swapchain_loader
        .create_swapchain(&swapchain_create_info, None)
        .unwrap();

    (swapchain, swapchain_loader)
}

unsafe fn create_swapchain_images(device: &ash::Device, swapchain: vk::SwapchainKHR,
            swapchain_loader: &Swapchain, surface_format: vk::SurfaceFormatKHR) -> (Vec<vk::Image>, Vec<vk::ImageView>) {

    let present_images = swapchain_loader.get_swapchain_images(swapchain).unwrap();
    let present_image_views: Vec<vk::ImageView> = present_images
        .iter()
        .map(|&image| {
            let create_view_info = vk::ImageViewCreateInfo::builder()
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(surface_format.format)
                .components(vk::ComponentMapping {
                    r: vk::ComponentSwizzle::R,
                    g: vk::ComponentSwizzle::G,
                    b: vk::ComponentSwizzle::B,
                    a: vk::ComponentSwizzle::A,
                })
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .image(image);
            device.create_image_view(&create_view_info, None).unwrap()
        })
        .collect();

    (present_images, present_image_views)
}

unsafe fn create_depth_image(device: &ash::Device, device_memory_properties: &vk::PhysicalDeviceMemoryProperties, surface_resolution: vk::Extent2D)
            -> (vk::Image, vk::DeviceMemory, vk::ImageView) {
    let depth_image_create_info = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::TYPE_2D)
        .format(vk::Format::D16_UNORM)
        .extent(vk::Extent3D {
            width: surface_resolution.width,
            height: surface_resolution.height,
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let depth_image = device.create_image(&depth_image_create_info, None).unwrap();
    let depth_image_memory_req = device.get_image_memory_requirements(depth_image);
    let depth_image_memory_index = find_memorytype_index(
        device_memory_properties,
        &depth_image_memory_req,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )
    .expect("Unable to find suitable memory index for depth image.");

    let depth_image_allocate_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(depth_image_memory_req.size)
        .memory_type_index(depth_image_memory_index);

    let depth_image_memory = device
        .allocate_memory(&depth_image_allocate_info, None)
        .unwrap();

    device
        .bind_image_memory(depth_image, depth_image_memory, 0)
        .expect("Unable to bind depth image memory");

    let depth_image_view_info = vk::ImageViewCreateInfo::builder()
        .subresource_range(
            vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::DEPTH)
                .level_count(1)
                .layer_count(1)
                .build(),
        )
        .image(depth_image)
        .format(depth_image_create_info.format)
        .view_type(vk::ImageViewType::TYPE_2D);

    let depth_image_view = device
        .create_image_view(&depth_image_view_info, None)
        .unwrap();

    (depth_image, depth_image_memory, depth_image_view)
}

// find the first suitable physical device and return it
unsafe fn find_physical_device(instance: &ash::Instance, pdevices: Vec<vk::PhysicalDevice>,
            surface_loader: &Surface, surface: vk::SurfaceKHR) -> (vk::PhysicalDevice, u32) {
    let (pdevice, queue_family_index) = pdevices.iter().map(
        | pdevice | {
            instance.get_physical_device_queue_family_properties(*pdevice)
                .iter()
                .enumerate()
                .find_map(|(index, info)| {
                    let supports_graphic_and_surface =
                        info.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                            && surface_loader.get_physical_device_surface_support(*pdevice, index as u32, surface).unwrap();
                    if supports_graphic_and_surface {
                        Some((*pdevice, index))
                    } else {
                        None
                    }
                })
        })
        .flatten()
        .next()
        .expect("Couldn't find suitable device.");
    (pdevice, queue_family_index as u32)
}

pub struct GraphicState {
    pub glfw: glfw::Glfw,
    pub window: glfw::Window,
    pub events: std::sync::mpsc::Receiver<(f64, glfw::WindowEvent)>,

    pub entry: ash::Entry,
    pub instance: ash::Instance,
    pub device: ash::Device,
    pub surface_loader: Surface,
    pub swapchain_loader: Swapchain,
    pub debug_utils_loader: DebugUtils,
    pub debug_call_back: vk::DebugUtilsMessengerEXT,

    pub pdevice: vk::PhysicalDevice,
    pub device_memory_properties: vk::PhysicalDeviceMemoryProperties,
    pub queue_family_index: u32,
    pub present_queue: vk::Queue,

    pub surface: vk::SurfaceKHR,
    pub surface_format: vk::SurfaceFormatKHR,
    pub surface_resolution: vk::Extent2D,

    // the swap chain is essentially a queue of images that are waiting to be presented to the screen. 
    pub swapchain: vk::SwapchainKHR,
    pub present_images: Vec<vk::Image>,
    pub present_image_views: Vec<vk::ImageView>,

    pub pool: vk::CommandPool,
    pub draw_command_buffer: vk::CommandBuffer,
    pub setup_command_buffer: vk::CommandBuffer,

    pub depth_image: vk::Image,
    pub depth_image_view: vk::ImageView,
    pub depth_image_memory: vk::DeviceMemory,

    pub present_complete_semaphore: vk::Semaphore,
    pub rendering_complete_semaphore: vk::Semaphore,

    pub draw_commands_reuse_fence: vk::Fence,
    pub setup_commands_reuse_fence: vk::Fence,
}

impl GraphicState {
    pub unsafe fn new(window_width: u32, window_height: u32) -> Self {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        // glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));

        let (mut window, events) = glfw.create_window(window_width, window_height, "Teardown", glfw::WindowMode::Windowed).expect("failed to create glfw window");

        window.make_current();
        window.set_key_polling(true);
        window.set_cursor_mode(glfw::CursorMode::Disabled);
        window.set_cursor_pos_polling(true);

        let entry = ash::Entry::linked();
        let instance = create_instance(&window, &entry);
        let (debug_utils_loader, debug_call_back) = create_debug_callback(&entry, &instance);

        let surface = ash_window::create_surface(&entry, &instance, window.raw_display_handle(), window.raw_window_handle(), None).unwrap();
        let surface_loader = Surface::new(&entry, &instance);

        let pdevices = instance.enumerate_physical_devices().expect("unable to list physical devices");
        let (pdevice, queue_family_index) = find_physical_device(&instance, pdevices, &surface_loader, surface);

        let device = create_device(&instance, pdevice, queue_family_index);

        let present_queue = device.get_device_queue(queue_family_index, 0);

        let surface_format = surface_loader.get_physical_device_surface_formats(pdevice, surface).unwrap()[0];

        let surface_capabilities = surface_loader.get_physical_device_surface_capabilities(pdevice, surface).unwrap();

        let surface_resolution = match surface_capabilities.current_extent.width {
            std::u32::MAX => vk::Extent2D {
                width: window_width,
                height: window_height,
            },
            _ => surface_capabilities.current_extent,
        };

        let (swapchain, swapchain_loader) = create_swapchain(&instance, &device, &pdevice, surface, &surface_capabilities, &surface_loader, surface_format, surface_resolution);
        let (present_images, present_image_views) = create_swapchain_images(&device, swapchain, &swapchain_loader, surface_format);

        let (pool, setup_command_buffer, draw_command_buffer) = create_command_buffers(&device, queue_family_index);

        let device_memory_properties = instance.get_physical_device_memory_properties(pdevice);

        let (depth_image, depth_image_memory, depth_image_view) = create_depth_image(&device, &device_memory_properties, surface_resolution);

        let semaphore_create_info = vk::SemaphoreCreateInfo::default();
        let present_complete_semaphore = device.create_semaphore(&semaphore_create_info, None).unwrap();
        let rendering_complete_semaphore = device.create_semaphore(&semaphore_create_info, None).unwrap();

        let fence_create_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
        let draw_commands_reuse_fence = device.create_fence(&fence_create_info, None).expect("Create fence failed.");
        let setup_commands_reuse_fence = device.create_fence(&fence_create_info, None).expect("Create fence failed.");

        submit_commandbuffer(
            &device,
            setup_command_buffer,
            setup_commands_reuse_fence,
            present_queue,
            &[],
            &[],
            &[],
            |device, setup_command_buffer| {
                let layout_transition_barriers = vk::ImageMemoryBarrier::builder()
                    .image(depth_image)
                    .dst_access_mask(
                        vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                            | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                    )
                    .new_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                    .old_layout(vk::ImageLayout::UNDEFINED)
                    .subresource_range(
                        vk::ImageSubresourceRange::builder()
                            .aspect_mask(vk::ImageAspectFlags::DEPTH)
                            .layer_count(1)
                            .level_count(1)
                            .build(),
                    );

                device.cmd_pipeline_barrier(
                    setup_command_buffer,
                    vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                    vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[layout_transition_barriers.build()],
                );
            },
        );

        GraphicState {
            glfw,
            window,
            events,
            entry,
            instance,
            device,
            queue_family_index,
            pdevice,
            device_memory_properties,
            surface_loader,
            surface_format,
            present_queue,
            surface_resolution,
            swapchain_loader,
            swapchain,
            present_images,
            present_image_views,
            pool,
            draw_command_buffer,
            setup_command_buffer,
            depth_image,
            depth_image_view,
            present_complete_semaphore,
            rendering_complete_semaphore,
            draw_commands_reuse_fence,
            setup_commands_reuse_fence,
            surface,
            debug_call_back,
            debug_utils_loader,
            depth_image_memory,
        }
    }
}

impl Drop for GraphicState {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().unwrap();
            self.device.destroy_semaphore(self.present_complete_semaphore, None);
            self.device.destroy_semaphore(self.rendering_complete_semaphore, None);
            self.device.destroy_fence(self.draw_commands_reuse_fence, None);
            self.device.destroy_fence(self.setup_commands_reuse_fence, None);
            self.device.free_memory(self.depth_image_memory, None);
            self.device.destroy_image_view(self.depth_image_view, None);
            self.device.destroy_image(self.depth_image, None);
            for &image_view in self.present_image_views.iter() {
                self.device.destroy_image_view(image_view, None);
            }
            self.device.destroy_command_pool(self.pool, None);
            self.swapchain_loader.destroy_swapchain(self.swapchain, None);
            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);
            self.debug_utils_loader.destroy_debug_utils_messenger(self.debug_call_back, None);
            self.instance.destroy_instance(None);
        }
    }
}
