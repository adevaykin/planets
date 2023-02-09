use std::mem;
use ash::vk;
use crate::engine::geometry::{Geometry, Vertex};
use crate::vulkan::device::Device;
use crate::vulkan::resources::manager::ResourceManager;

pub fn create_as(device: &Device, resource_manager: &mut ResourceManager, geometry: &Geometry) {
    let acceleration_structure =
        ash::extensions::khr::AccelerationStructure::new(&device.instance.instance, &device.logical_device);

    let as_geometry = vk::AccelerationStructureGeometryKHR::builder()
        .geometry_type(vk::GeometryTypeKHR::TRIANGLES)
        .geometry(vk::AccelerationStructureGeometryDataKHR {
            triangles: vk::AccelerationStructureGeometryTrianglesDataKHR::builder()
                .vertex_data(vk::DeviceOrHostAddressConstKHR {
                    device_address: device.get_buffer_device_address(geometry.vertex_buffer.borrow().buffer),
                })
                .max_vertex(geometry.vertices.len() as u32 - 1)
                .vertex_stride(mem::size_of::<Vertex>() as u64)
                .vertex_format(vk::Format::R32G32B32_SFLOAT)
                .index_data(vk::DeviceOrHostAddressConstKHR {
                    device_address: device.get_buffer_device_address(geometry.index_buffer.borrow().buffer),
                })
                .index_type(vk::IndexType::UINT32)
                .build(),
        })
        .flags(vk::GeometryFlagsKHR::OPAQUE)
        .build();

    let (bottom_as, bottom_as_buffer) = {
        let build_range_info = vk::AccelerationStructureBuildRangeInfoKHR::builder()
            .first_vertex(0)
            .primitive_count(geometry.indices.len() as u32 / 3)
            .primitive_offset(0)
            .transform_offset(0)
            .build();

        let geometries = [as_geometry];

        let mut build_info = vk::AccelerationStructureBuildGeometryInfoKHR::builder()
            .flags(vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE)
            .geometries(&geometries)
            .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
            .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL)
            .build();

        let size_info = unsafe {
            acceleration_structure.get_acceleration_structure_build_sizes(
                vk::AccelerationStructureBuildTypeKHR::DEVICE,
                &build_info,
                &[geometry.indices.len() as u32 / 3],
            )
        };

        let bottom_as_buffer = resource_manager.buffer_with_size(
            size_info.acceleration_structure_size,
            vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
                | vk::BufferUsageFlags::STORAGE_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            "AS Bottom Buffer"
        );

        let as_create_info = vk::AccelerationStructureCreateInfoKHR::builder()
            .ty(build_info.ty)
            .size(size_info.acceleration_structure_size)
            .buffer(bottom_as_buffer.borrow().buffer)
            .offset(0)
            .build();

        let bottom_as =
            unsafe { acceleration_structure.create_acceleration_structure(&as_create_info, None) }
                .unwrap();

        build_info.dst_acceleration_structure = bottom_as;

        let scratch_buffer = resource_manager.buffer_with_size(
            size_info.build_scratch_size,
            vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS | vk::BufferUsageFlags::STORAGE_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            "AS Scratch Buffer",
        );

        build_info.scratch_data = vk::DeviceOrHostAddressKHR {
            device_address: unsafe { device.get_buffer_device_address(scratch_buffer.borrow().buffer) },
        };

        let command_buffer = device.get_command_buffer();
        unsafe {
            acceleration_structure.cmd_build_acceleration_structures(
                command_buffer,
                &[build_info],
                &[&[build_range_info]]
            );
        }

        (bottom_as, bottom_as_buffer)
    };

    // let (top_as, top_as_buffer) = {
    //     let build_range_info = vk::AccelerationStructureBuildRangeInfoKHR::builder()
    //         .first_vertex(0)
    //         .primitive_count(instance_count as u32)
    //         .primitive_offset(0)
    //         .transform_offset(0)
    //         .build();
    //
    //     let build_command_buffer = {
    //         let allocate_info = vk::CommandBufferAllocateInfo::builder()
    //             .command_buffer_count(1)
    //             .command_pool(command_pool)
    //             .level(vk::CommandBufferLevel::PRIMARY)
    //             .build();
    //
    //         let command_buffers =
    //             unsafe { device.allocate_command_buffers(&allocate_info) }.unwrap();
    //         command_buffers[0]
    //     };
    //
    //     unsafe {
    //         device
    //             .begin_command_buffer(
    //                 build_command_buffer,
    //                 &vk::CommandBufferBeginInfo::builder()
    //                     .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
    //                     .build(),
    //             )
    //             .unwrap();
    //         let memory_barrier = vk::MemoryBarrier::builder()
    //             .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
    //             .dst_access_mask(vk::AccessFlags::ACCELERATION_STRUCTURE_WRITE_KHR)
    //             .build();
    //         device.cmd_pipeline_barrier(
    //             build_command_buffer,
    //             vk::PipelineStageFlags::TRANSFER,
    //             vk::PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR,
    //             vk::DependencyFlags::empty(),
    //             &[memory_barrier],
    //             &[],
    //             &[],
    //         );
    //     }
    //
    //     let instances = vk::AccelerationStructureGeometryInstancesDataKHR::builder()
    //         .array_of_pointers(false)
    //         .data(vk::DeviceOrHostAddressConstKHR {
    //             device_address: unsafe {
    //                 get_buffer_device_address(&device, instance_buffer.buffer)
    //             },
    //         })
    //         .build();
    //
    //     let geometry = vk::AccelerationStructureGeometryKHR::builder()
    //         .geometry_type(vk::GeometryTypeKHR::INSTANCES)
    //         .geometry(vk::AccelerationStructureGeometryDataKHR { instances })
    //         .build();
    //
    //     let geometries = [geometry];
    //
    //     let mut build_info = vk::AccelerationStructureBuildGeometryInfoKHR::builder()
    //         .flags(vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE)
    //         .geometries(&geometries)
    //         .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
    //         .ty(vk::AccelerationStructureTypeKHR::TOP_LEVEL)
    //         .build();
    //
    //     let size_info = unsafe {
    //         acceleration_structure.get_acceleration_structure_build_sizes(
    //             vk::AccelerationStructureBuildTypeKHR::DEVICE,
    //             &build_info,
    //             &[build_range_info.primitive_count],
    //         )
    //     };
    //
    //     let top_as_buffer = BufferResource::new(
    //         size_info.acceleration_structure_size,
    //         vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
    //             | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
    //             | vk::BufferUsageFlags::STORAGE_BUFFER,
    //         vk::MemoryPropertyFlags::DEVICE_LOCAL,
    //         &device,
    //         device_memory_properties,
    //     );
    //
    //     let as_create_info = vk::AccelerationStructureCreateInfoKHR::builder()
    //         .ty(build_info.ty)
    //         .size(size_info.acceleration_structure_size)
    //         .buffer(top_as_buffer.buffer)
    //         .offset(0)
    //         .build();
    //
    //     let top_as =
    //         unsafe { acceleration_structure.create_acceleration_structure(&as_create_info, None) }
    //             .unwrap();
    //
    //     build_info.dst_acceleration_structure = top_as;
    //
    //     let scratch_buffer = BufferResource::new(
    //         size_info.build_scratch_size,
    //         vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS | vk::BufferUsageFlags::STORAGE_BUFFER,
    //         vk::MemoryPropertyFlags::DEVICE_LOCAL,
    //         &device,
    //         device_memory_properties,
    //     );
    //
    //     build_info.scratch_data = vk::DeviceOrHostAddressKHR {
    //         device_address: unsafe { get_buffer_device_address(&device, scratch_buffer.buffer) },
    //     };
    //
    //     unsafe {
    //         acceleration_structure.cmd_build_acceleration_structures(
    //             build_command_buffer,
    //             &[build_info],
    //             &[&[build_range_info]],
    //         );
    //         device.end_command_buffer(build_command_buffer).unwrap();
    //         device
    //             .queue_submit(
    //                 graphics_queue,
    //                 &[vk::SubmitInfo::builder()
    //                     .command_buffers(&[build_command_buffer])
    //                     .build()],
    //                 vk::Fence::null(),
    //             )
    //             .expect("queue submit failed.");
    //
    //         device.queue_wait_idle(graphics_queue).unwrap();
    //         device.free_command_buffers(command_pool, &[build_command_buffer]);
    //         scratch_buffer.destroy(&device);
    //     }
    //
    //     (top_as, top_as_buffer)
    // };
}