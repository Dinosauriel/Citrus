use ash::vk;
use ash::util::Align;
use std::mem::align_of;
use crate::*;

pub struct Buffer<'a> {
    device: &'a ash::Device,
    pub vk_buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    memory_requirements: vk::MemoryRequirements,
}

impl<'l> Buffer<'l> {
    pub unsafe fn create(
            device: &'l ash::Device, device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
            size: vk::DeviceSize, usage: vk::BufferUsageFlags, properties: vk::MemoryPropertyFlags) 
                -> Self {
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = device.create_buffer(&buffer_info, None).unwrap();
        let memory_req = device.get_buffer_memory_requirements(buffer);

        let memory_index = find_memorytype_index(&device_memory_properties, &memory_req, properties)
        .expect("unable to find suitable memorytype for buffer.");

        let allocation_info = vk::MemoryAllocateInfo {
            allocation_size: memory_req.size,
            memory_type_index: memory_index,
            ..Default::default()
        };

        let memory = device.allocate_memory(&allocation_info, None).unwrap();
        // let mapped_memory_pointer = device.map_memory(memory, 0, memory_req.size, vk::MemoryMapFlags::empty()).unwrap();

        device.bind_buffer_memory(buffer, memory, 0).unwrap();

        return Buffer {
            device,
            vk_buffer: buffer,
            memory,
            memory_requirements: memory_req
        }
    }

    pub unsafe fn fill<T: std::marker::Copy>(&self, content: &[T]) {
        let pointer = self.device.map_memory(self.memory, 0, self.memory_requirements.size, vk::MemoryMapFlags::empty()).unwrap();
        let mut align = Align::new(pointer, align_of::<T>() as u64, self.memory_requirements.size);
        align.copy_from_slice(&content);
        self.device.unmap_memory(self.memory);
    }

    pub unsafe fn free(&self, device: &ash::Device) {
        device.free_memory(self.memory, None);
        device.destroy_buffer(self.vk_buffer, None);
    }
}
