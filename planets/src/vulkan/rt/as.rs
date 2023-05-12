use std::mem;
use ash::extensions::khr;
use ash::util::Align;
use ash::vk;
use crate::engine::geometry::{Geometry, Vertex};
use crate::vulkan::cmd_buffers::SingleTimeCmdBuffer;
use crate::vulkan::device::Device;
use crate::vulkan::mem::{AllocatedBufferMutRef, BufferAccess, VecBufferData};
use crate::vulkan::resources::manager::ResourceManager;

pub struct AccelerationStructure {
    accel: khr::AccelerationStructure,
    pub blas: vk::AccelerationStructureKHR,
    blas_buffer: AllocatedBufferMutRef,
    pub tlas: vk::AccelerationStructureKHR,
    tlas_buffer: AllocatedBufferMutRef,
}

impl AccelerationStructure {
    pub fn new(device: &Device, resource_manager: &mut ResourceManager, geometries: &[Geometry]) -> Self {
        let acceleration_structure =
            khr::AccelerationStructure::new(&device.instance.instance, &device.logical_device);

        let mut as_geometries = vec![];
        let mut as_geometry_counts = vec![];
        let mut as_build_range_infos = vec![];
        for geometry in geometries {
            let as_geometry = vk::AccelerationStructureGeometryKHR::builder()
                .geometry_type(vk::GeometryTypeKHR::TRIANGLES)
                .geometry(vk::AccelerationStructureGeometryDataKHR {
                    triangles: vk::AccelerationStructureGeometryTrianglesDataKHR::builder()
                        .vertex_data(vk::DeviceOrHostAddressConstKHR {
                            device_address: geometry.vertex_buffer.borrow().get_buffer_device_address(device),
                        })
                        .max_vertex(geometry.vertices.len() as u32 - 1)
                        .vertex_stride(mem::size_of::<Vertex>() as u64)
                        .vertex_format(vk::Format::R32G32B32_SFLOAT)
                        .index_data(vk::DeviceOrHostAddressConstKHR {
                            device_address: geometry.index_buffer.borrow().get_buffer_device_address(device),
                        })
                        .index_type(vk::IndexType::UINT32)
                        .build(),
                })
                .flags(vk::GeometryFlagsKHR::OPAQUE)
                .build();
            as_geometries.push(as_geometry);

            as_geometry_counts.push(geometry.get_primitives_count());

            let as_build_range_info = vk::AccelerationStructureBuildRangeInfoKHR::builder()
                .first_vertex(0)
                .primitive_count(geometry.get_primitives_count())
                .primitive_offset(0)
                .transform_offset(0)
                .build();
            as_build_range_infos.push(as_build_range_info);
        }

        let (bottom_as, bottom_as_buffer) = {
            let mut build_info = vk::AccelerationStructureBuildGeometryInfoKHR::builder()
                .flags(vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE)
                .geometries(&as_geometries)
                .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
                .ty(vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL)
                .build();

            let size_info = unsafe {
                acceleration_structure.get_acceleration_structure_build_sizes(
                    vk::AccelerationStructureBuildTypeKHR::DEVICE,
                    &build_info,
                    &as_geometry_counts,
                )
            };

            let bottom_as_buffer = resource_manager.buffer_with_size(
                size_info.acceleration_structure_size,
                vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                    | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
                    | vk::BufferUsageFlags::STORAGE_BUFFER,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
                "BLAS Buffer"
            );

            let bottom_as_vk_buffer = {
                let barrier_params = BufferAccess {
                    src_access: vk::AccessFlags::TRANSFER_WRITE,
                    src_stage: vk::PipelineStageFlags::TRANSFER,
                    dst_access: vk::AccessFlags::ACCELERATION_STRUCTURE_WRITE_KHR,
                    dst_stage: vk::PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR,
                };

                bottom_as_buffer.borrow().access_buffer(device, &barrier_params)
            };

            let as_create_info = vk::AccelerationStructureCreateInfoKHR::builder()
                .ty(build_info.ty)
                .size(size_info.acceleration_structure_size)
                .buffer(bottom_as_vk_buffer)
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
                "BLAS Scratch Buffer",
            );

            build_info.scratch_data = vk::DeviceOrHostAddressKHR {
                device_address: scratch_buffer.borrow().get_buffer_device_address(device),
            };

            let command_buffer = SingleTimeCmdBuffer::begin(device);
            unsafe {
                acceleration_structure.cmd_build_acceleration_structures(
                    command_buffer.get_command_buffer(),
                    &[build_info],
                    &[&as_build_range_infos[0..]]
                );
            }

            (bottom_as, bottom_as_buffer)
        };

        let accel_handle = {
            let as_addr_info = vk::AccelerationStructureDeviceAddressInfoKHR::builder()
                .acceleration_structure(bottom_as)
                .build();
            unsafe { acceleration_structure.get_acceleration_structure_device_address(&as_addr_info) }
        };

        let (instance_count, instance_buffer) = {
            let transform_0: [f32; 12] = [1.0, 0.0, 0.0, -1.5, 0.0, 1.0, 0.0, 1.1, 0.0, 0.0, 1.0, 0.0];

            let instances = vec![
                vk::AccelerationStructureInstanceKHR {
                    transform: vk::TransformMatrixKHR {
                        matrix: transform_0,
                    },
                    instance_custom_index_and_mask: vk::Packed24_8::new(0xff << 24, 0),
                    instance_shader_binding_table_record_offset_and_flags: vk::Packed24_8::new(vk::GeometryInstanceFlagsKHR::TRIANGLE_FACING_CULL_DISABLE.as_raw() << 24, 0),
                    acceleration_structure_reference: vk::AccelerationStructureReferenceKHR {
                        device_handle: accel_handle,
                    },
                },
            ];

            let instance_data = VecBufferData::new(&instances);
            let instance_buffer = resource_manager.buffer_host_visible_coherent(
                &instance_data,
                vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR
                    | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
                "TLAS Instance Buffer"
            );

            (instances.len() as u32, instance_buffer)
        };

        let (top_as, top_as_buffer) = {
            let build_range_info = vk::AccelerationStructureBuildRangeInfoKHR::builder()
                .first_vertex(0)
                .primitive_count(instance_count)
                .primitive_offset(0)
                .transform_offset(0)
                .build();

            unsafe {
                let memory_barrier = vk::MemoryBarrier::builder()
                    .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                    .dst_access_mask(vk::AccessFlags::ACCELERATION_STRUCTURE_WRITE_KHR)
                    .build();
                device.logical_device.cmd_pipeline_barrier(
                    device.get_command_buffer(),
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR,
                    vk::DependencyFlags::empty(),
                    &[memory_barrier],
                    &[],
                    &[],
                );
            }

            let instances = vk::AccelerationStructureGeometryInstancesDataKHR::builder()
                .array_of_pointers(false)
                .data(vk::DeviceOrHostAddressConstKHR {
                    device_address: instance_buffer.borrow().get_buffer_device_address(device),
                })
                .build();

            let geometry = vk::AccelerationStructureGeometryKHR::builder()
                .geometry_type(vk::GeometryTypeKHR::INSTANCES)
                .geometry(vk::AccelerationStructureGeometryDataKHR { instances })
                .build();

            let geometries = [geometry];

            let mut build_info = vk::AccelerationStructureBuildGeometryInfoKHR::builder()
                .flags(vk::BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE)
                .geometries(&geometries)
                .mode(vk::BuildAccelerationStructureModeKHR::BUILD)
                .ty(vk::AccelerationStructureTypeKHR::TOP_LEVEL)
                .build();

            let size_info = unsafe {
                acceleration_structure.get_acceleration_structure_build_sizes(
                    vk::AccelerationStructureBuildTypeKHR::DEVICE,
                    &build_info,
                    &[build_range_info.primitive_count],
                )
            };

            let top_as_buffer = resource_manager.buffer_with_size(
                size_info.acceleration_structure_size,
                vk::BufferUsageFlags::ACCELERATION_STRUCTURE_STORAGE_KHR
                    | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
                    | vk::BufferUsageFlags::STORAGE_BUFFER,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
                "TLAS Buffer"
            );

            let top_as_vk_buffer = {
                let barrier_params = BufferAccess {
                    src_access: vk::AccessFlags::TRANSFER_WRITE,
                    src_stage: vk::PipelineStageFlags::TRANSFER,
                    dst_access: vk::AccessFlags::ACCELERATION_STRUCTURE_WRITE_KHR,
                    dst_stage: vk::PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR,
                };

                top_as_buffer.borrow().access_buffer(device, &barrier_params)
            };

            let as_create_info = vk::AccelerationStructureCreateInfoKHR::builder()
                .ty(build_info.ty)
                .size(size_info.acceleration_structure_size)
                .buffer(top_as_vk_buffer)
                .offset(0)
                .build();

            let top_as =
                unsafe { acceleration_structure.create_acceleration_structure(&as_create_info, None) }
                    .unwrap();

            build_info.dst_acceleration_structure = top_as;

            let scratch_buffer = resource_manager.buffer_with_size(
                size_info.build_scratch_size,
                vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS | vk::BufferUsageFlags::STORAGE_BUFFER,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
                "TLAS Scratch Buffer",
            );

            build_info.scratch_data = vk::DeviceOrHostAddressKHR {
                device_address: scratch_buffer.borrow().get_buffer_device_address(device),
            };

            let command_buffer = SingleTimeCmdBuffer::begin(device);
            unsafe {
                acceleration_structure.cmd_build_acceleration_structures(
                    command_buffer.get_command_buffer(),
                    &[build_info],
                    &[&[build_range_info]],
                );
            }

            (top_as, top_as_buffer)
        };

        AccelerationStructure {
            accel: acceleration_structure,
            blas: bottom_as,
            blas_buffer: bottom_as_buffer,
            tlas: top_as,
            tlas_buffer: top_as_buffer,
        }
    }
}