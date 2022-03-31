use ash::vk;
use ash::util::Align;
use std::mem::align_of;
use crate::*;

pub struct Buffer {
    pub vulkan_instance: vk::Buffer,
    memory: vk::DeviceMemory,
    memory_requirements: vk::MemoryRequirements,
}

impl Buffer {
    pub unsafe fn create(device: &ash::Device, device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
        size: vk::DeviceSize, usage: vk::BufferUsageFlags, properties: vk::MemoryPropertyFlags) 
        -> Self {
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

        return Buffer {
            vulkan_instance: buffer,
            memory: memory,
            memory_requirements: memory_req
        }
    }

    pub unsafe fn fill<T: std::marker::Copy>(&self, device: &ash::Device, content: &[T]) {
        let pointer = device.map_memory(self.memory, 0, self.memory_requirements.size, vk::MemoryMapFlags::empty()).unwrap();
        let mut align = Align::new(pointer, align_of::<T>() as u64, self.memory_requirements.size);
        align.copy_from_slice(&content);
        device.unmap_memory(self.memory);
    }

    pub unsafe fn free(&self, device: &ash::Device) {
        device.free_memory(self.memory, None);
        device.destroy_buffer(self.vulkan_instance, None);
    }
}
