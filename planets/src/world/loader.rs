use std::path::Path;
use std::io::{Seek,SeekFrom,Read};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use cgmath as cgm;

use crate::engine::geometry::{Geometry, Vertex};
use crate::engine::material::Material;
use crate::engine::scene::node::{Node, NodeContent, NodeMutRef};
use crate::engine::textures::TextureManagerMutRef;

use crate::vulkan::drawable::{Drawable, DrawType};
use crate::vulkan::resources::ResourceManagerMutRef;

pub struct ModelLoader {
    resource_manager: ResourceManagerMutRef,
    texture_manager: TextureManagerMutRef,
    loaded_models: HashMap<String,NodeMutRef>,
}

impl ModelLoader {
    pub fn new(resource_manager: &ResourceManagerMutRef, texture_manager: &TextureManagerMutRef) -> ModelLoader {
        ModelLoader {
            resource_manager: Rc::clone(resource_manager),
            texture_manager: Rc::clone(texture_manager),
            loaded_models: HashMap::new(),
        }
    }

    pub fn load_gltf(&mut self, path: &str) -> NodeMutRef {
        match self.loaded_models.get(path) {
            Some(x) => return x.borrow().create_instance(),
            None => {
                let node = Rc::new(RefCell::new(Node::new()));
                if self.load_gltf_impl(path, &node) {
                    self.loaded_models.insert(path.to_string(), Rc::clone(&node));
                    return self.load_gltf(path);
                } else {
                    log::error!("Failed to load model {}", path);
                    return Rc::new(RefCell::new(Node::new()));
                }
            }
        }
    }

    pub fn load_gltf_impl(&mut self, path: &str, node: &NodeMutRef) -> bool {
        let (document, _, _) = match gltf::import(path) {
            Ok(x) => x,
            Err(x) => {
                log::error!("Failed to read {} model: {}", path, x);
                return false;
            }
        };

        if document.scenes().count() != 1 {
            log::warn!("Number of scenes ({}) in gltf file not supported. Only first scene will be loaded.", document.scenes().count());
        }
        let gltf_dir_path = Path::new(path).parent().expect("Failed to get gltf file directory path.");
        let scene = document.scenes().next().expect("Failed to unwrap scene.");
        if scene.nodes().len() > 1 {
            log::warn!("GLTF file contains {} nodes. Only one will be loaded.", scene.nodes().len());
        }
        for n in scene.nodes() {
            self.node_from_gltf(node, &n, &gltf_dir_path);
            return true;
        }

        false
    }

    fn node_from_gltf(&mut self, node: &NodeMutRef, gltf_node: &gltf::Node, gltf_dir_path: &Path) {
        let gltf_transform = gltf_node.transform().matrix();
        let transform = cgm::Matrix4 {
            x: cgm::Vector4 { x: gltf_transform[0][0], y: gltf_transform[0][1], z: gltf_transform[0][2], w: gltf_transform[0][3] },
            y: cgm::Vector4 { x: gltf_transform[1][0], y: gltf_transform[1][1], z: gltf_transform[1][2], w: gltf_transform[1][3] },
            z: cgm::Vector4 { x: gltf_transform[2][0], y: gltf_transform[2][1], z: gltf_transform[2][2], w: gltf_transform[2][3] },
            w: cgm::Vector4 { x: gltf_transform[3][0], y: gltf_transform[3][1], z: gltf_transform[3][2], w: gltf_transform[3][3] }
        };
        let transform_node = Rc::new(RefCell::new(Node::with_content(NodeContent::Transform(transform))));

        if gltf_node.mesh().is_some() {
            let mesh = gltf_node.mesh().unwrap();
            for primitive in mesh.primitives() {
                if primitive.mode() != gltf::mesh::Mode::Triangles {
                    log::info!("Unsupported mesh primitive mode.");
                    continue;
                }
                let semantic = gltf::mesh::Semantic::Positions;
                let accessor = primitive.get(&semantic).expect("Mesh position attribute is missing.");
                if accessor.size() != std::mem::size_of::<cgm::Vector3<f32>>() {
                    panic!("Mesh position attribute element size is not of size vec3!");
                }
                let positions_data: Vec<cgm::Vector3<f32>> = ModelLoader::read_from_gltf_accessor(&accessor, gltf_dir_path);

                let semantic = gltf::mesh::Semantic::Normals;
                let accessor = primitive.get(&semantic).expect("Mesh normal attribute is missing.");
                if accessor.size() != std::mem::size_of::<cgm::Vector3<f32>>() {
                    panic!("Mesh normal attribute element size is not of size vec3!");
                }
                let normal_data: Vec<cgm::Vector3<f32>> = ModelLoader::read_from_gltf_accessor(&accessor, gltf_dir_path);

                let semantic = gltf::mesh::Semantic::TexCoords(0);
                let accessor = primitive.get(&semantic).expect("Mesh uv attribute is missing.");
                if accessor.size() != std::mem::size_of::<cgm::Vector2<f32>>() {
                    panic!("Mesh uv attribute element size is not of size vec2!");
                }
                let uv_data: Vec<cgm::Vector2<f32>> = ModelLoader::read_from_gltf_accessor(&accessor, gltf_dir_path);

                let accessor = primitive.indices().expect("Could not get indices accessor.");
                if accessor.size() != std::mem::size_of::<cgm::Vector1<u16>>() {
                    panic!("Mesh indices attribute element size is not of size u16");
                }
                let indices_data: Vec<cgm::Vector1<u16>> = ModelLoader::read_from_gltf_accessor(&accessor, gltf_dir_path);

                let indices = indices_data.into_iter().map(|idx| idx.x as u32).collect();
                let mut vertices = vec![];
                vertices.resize(positions_data.len(), Vertex::from_position(0.0, 0.0, 0.0,));
                for i in 0..positions_data.len() {
                    vertices[i].position = positions_data[i];
                    vertices[i].normal = normal_data[i];
                    vertices[i].uv = uv_data[i];
                }
                let material = self.material_from_gltf(gltf_dir_path, primitive.material());
                let mut resource_manager = self.resource_manager.borrow_mut();
                let geometry = Geometry::new(&mut resource_manager, vertices, indices);
                let drawable = Rc::new(RefCell::new(Drawable::new(DrawType::Opaque, geometry, material)));
                let drawable_node= Rc::new(RefCell::new(Node::with_content(NodeContent::Drawable(drawable))));
                transform_node.borrow_mut().add_child(drawable_node);
                node.borrow_mut().add_child(Rc::clone(&transform_node));
            }
        }
    }

    fn read_from_gltf_accessor<T>(accessor: &gltf::Accessor, gltf_dir_path: &Path) -> Vec<T>
        where T: Clone + cgm::Zero {
        let view = accessor.view().expect("Could not get a view into gltf buffer.");
        let buf = view.buffer();
        let data: Vec<T> = match buf.source() {
            gltf::buffer::Source::Uri(path) => {
                let full_path = gltf_dir_path.join(path);
                ModelLoader::read_from_file(&full_path, view.offset() as u64, view.length() as u64)
            },
            _ => {
                panic!("Buffer source type BIN not supported.");
            }
        };

        data
    }

    fn read_from_file<T>(path: &Path, offset: u64, byte_count: u64) -> Vec<T>
        where T: Clone + cgm::Zero {
        let mut file = match std::fs::File::open(path) {
            Ok(x) => x,
            Err(_) => {
                log::error!("Could not read file {}", path.to_str().unwrap_or("UNKNOWN"));
                panic!();
            }
        };

        file.seek(SeekFrom::Start(offset)).expect("Failed to seek to the given offset in the buffer file.");

        let vec_len = byte_count / std::mem::size_of::<T>() as u64;
        let mut data = vec![];
        data.reserve(vec_len as usize);
        for _ in 0..vec_len {
            unsafe {
                let mut element = T::zero();
                let mut data_slice = std::slice::from_raw_parts_mut(&mut element as *mut _ as *mut u8, std::mem::size_of::<T>());
                file.read_exact(&mut data_slice).expect("Failed to read wanted number of bytes from file.");
                data.push(element);
            }
        }

        data
    }

    fn material_from_gltf(&mut self, gltf_dir_path: &Path, gltf_material: gltf::Material) -> Material {
        let mut material = Material::new();
        let pbr_metallic_roughness = gltf_material.pbr_metallic_roughness();
        let base_color_texture = pbr_metallic_roughness.base_color_texture().expect("Expected PBR base color texture.");
        let texture = base_color_texture.texture();
        let image = texture.source();
        let image_path = match image.source() {
            gltf::image::Source::Uri{uri, mime_type: _} => uri,
            _ => panic!("Image view is not supported (yet).")
        };

        let full_path = gltf_dir_path.join(image_path);
        material.albedo_map = Some(
            Rc::clone(
                self.texture_manager.borrow_mut().get_texture(
                    full_path.to_str().expect("Could not unwrap gltf image full path")
                )
            )
        );

        material
    }
}
