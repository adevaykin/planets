use std::path::Path;
use std::io::{Seek,SeekFrom,Read};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use cgmath as cgm;

use crate::engine::geometry::{Geometry, Vertex};
use crate::engine::material::Material;
use crate::engine::scene::node::{Node, NodeContent, NodeMutRef};
use crate::engine::textures::{TextureManager, TextureManagerMutRef};

use crate::vulkan::drawable::{Drawable, DrawType};
use crate::vulkan::resources::{ResourceManager, ResourceManagerMutRef};

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

    pub fn load_gltf(&mut self, path: &str) -> Result<NodeMutRef, String> {
        match self.loaded_models.get(path) {
            Some(x) => {
                Ok(Rc::clone(x))
            },
            None => {
                match self.load_gltf_impl(path) {
                    Ok(nodes) => {
                        let loaded_model = if nodes.len() > 1 {
                            let mut group = Node::with_content(NodeContent::Group);
                            for n in &nodes {
                                group.add_child(Rc::clone(n));
                            }
                            Rc::new(RefCell::new(group))
                        } else {
                            Rc::clone(&nodes[0])
                        };
                        self.loaded_models.insert(path.to_string(), loaded_model);
                        self.load_gltf(path)
                    },
                    Err(str) => Err(str)
                }
            }
        }
    }

    pub fn load_gltf_impl(&mut self, path: &str) -> Result<Vec<NodeMutRef>,String> {
        let (document, _, _) = match gltf::import(path) {
            Ok(x) => x,
            Err(e) => {
                return Err(format!("Failed to read {} model: {}", path, e))
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

        let mut res = vec![];
        for n in scene.nodes() {
            match self.node_from_gltf(&n, gltf_dir_path) {
                Ok(nodes) => {
                    for n in &nodes {
                        res.push(Rc::clone(n))
                    }
                },
                Err(str) => {
                    log::warn!("Failed to load scene node from gltf with the following reason:");
                    log::error!("{}", str);
                }
            }
        }
        Ok(res)
    }

    fn node_from_gltf(&mut self, gltf_node: &gltf::Node, gltf_dir_path: &Path) -> Result<Vec<NodeMutRef>,String> {
        if let Some(mesh) = gltf_node.mesh() {
            // TODO: don't create transform node if node transform is identity
            let gltf_transform = gltf_node.transform().matrix();
            let transform = cgm::Matrix4 {
                x: cgm::Vector4 { x: gltf_transform[0][0], y: gltf_transform[0][1], z: gltf_transform[0][2], w: gltf_transform[0][3] },
                y: cgm::Vector4 { x: gltf_transform[1][0], y: gltf_transform[1][1], z: gltf_transform[1][2], w: gltf_transform[1][3] },
                z: cgm::Vector4 { x: gltf_transform[2][0], y: gltf_transform[2][1], z: gltf_transform[2][2], w: gltf_transform[2][3] },
                w: cgm::Vector4 { x: gltf_transform[3][0], y: gltf_transform[3][1], z: gltf_transform[3][2], w: gltf_transform[3][3] }
            };
            let transform_node = Rc::new(RefCell::new(Node::with_content(NodeContent::Transform(transform))));
            match ModelLoader::mesh_from_node(
                    &mut self.resource_manager.borrow_mut(),
                    &mut self.texture_manager.borrow_mut(),
                    &mesh,
                    gltf_dir_path) {
                Ok(mesh_node) => transform_node.borrow_mut().add_child(mesh_node),
                Err(str) => {
                    log::warn!("Could not load mesh node from {}", gltf_dir_path.to_str().unwrap_or("Unknown"));
                    log::error!("{}", str);
                },
            };

            for child in gltf_node.children() {
                if let Ok(children_nodes) = self.node_from_gltf(&child, gltf_dir_path) {
                    for c in &children_nodes {
                        transform_node.borrow_mut().add_child(Rc::clone(c));
                    }
                }
            }

            Ok(vec![transform_node])
        } else {
            let mut children = vec![];
            for child in gltf_node.children() {
                if let Ok(children_nodes) = self.node_from_gltf(&child, gltf_dir_path) {
                    for c in &children_nodes {
                        children.push(Rc::clone(c));
                    }
                }
            }

            if !children.is_empty() {
                Ok(children)
            } else {
                Err(String::from("Didn't find any supported nodes in gltf"))
            }
        }
    }

    fn mesh_from_node(resource_manager: &mut ResourceManager, texture_manager: &mut TextureManager, mesh: &gltf::Mesh, gltf_dir_path: &Path) -> Result<NodeMutRef,String> {
        for primitive in mesh.primitives() {
            if primitive.mode() != gltf::mesh::Mode::Triangles {
                log::warn!("Unsupported mesh primitive mode.");
                continue;
            }
            let semantic = gltf::mesh::Semantic::Positions;
            let accessor = primitive.get(&semantic).expect("Mesh position attribute is missing.");
            if accessor.size() != std::mem::size_of::<cgm::Vector3<f32>>() {
                return Err(String::from("Mesh position attribute element size is not of size vec3!"));
            }
            let positions_data: Vec<cgm::Vector3<f32>> = ModelLoader::read_from_gltf_accessor(&accessor, gltf_dir_path);

            let semantic = gltf::mesh::Semantic::Normals;
            let accessor = primitive.get(&semantic).expect("Mesh normal attribute is missing.");
            if accessor.size() != std::mem::size_of::<cgm::Vector3<f32>>() {
                return Err(String::from("Mesh normal attribute element size is not of size vec3!"));
            }
            let normal_data: Vec<cgm::Vector3<f32>> = ModelLoader::read_from_gltf_accessor(&accessor, gltf_dir_path);

            let semantic = gltf::mesh::Semantic::TexCoords(0);
            let uv_data: Vec<cgm::Vector2<f32>> = if let Some(accessor) = primitive.get(&semantic) {
                if accessor.size() != std::mem::size_of::<cgm::Vector2<f32>>() {
                    log::warn!("Mesh uv attribute element size is not of size vec2!");
                    vec![cgm::Vector2::new(0.0, 0.0); positions_data.len()]
                } else {
                    ModelLoader::read_from_gltf_accessor(&accessor, gltf_dir_path)
                }
            } else {
                vec![cgm::Vector2::new(0.0, 0.0); positions_data.len()]
            };

            let accessor = primitive.indices().expect("Could not get indices accessor.");
            if accessor.size() != std::mem::size_of::<cgm::Vector1<u16>>() {
                return Err(String::from("Mesh indices attribute element size is not of size u16"));
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
            let material = ModelLoader::material_from_gltf(texture_manager, gltf_dir_path, primitive.material());
            let geometry = Geometry::new(resource_manager, vertices, indices);
            let drawable = Rc::new(RefCell::new(Drawable::new(DrawType::Opaque, geometry, material)));
            let drawable_node= Rc::new(RefCell::new(Node::with_content(NodeContent::Drawable(drawable))));

            return Ok(drawable_node);
        }

        Err(String::from("No primitives in mesh"))
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
                let data_slice = std::slice::from_raw_parts_mut(&mut element as *mut _ as *mut u8, std::mem::size_of::<T>());
                file.read_exact(data_slice).expect("Failed to read wanted number of bytes from file.");
                data.push(element);
            }
        }

        data
    }

    fn material_from_gltf(texture_manager: &mut TextureManager, gltf_dir_path: &Path, gltf_material: gltf::Material) -> Material {
        let mut material = Material::new();
        let pbr_metallic_roughness = gltf_material.pbr_metallic_roughness();
        if let Some(base_color_texture) = pbr_metallic_roughness.base_color_texture() {
            let texture = base_color_texture.texture();
            let image = texture.source();
            let image_path = match image.source() {
                gltf::image::Source::Uri { uri, mime_type: _ } => uri,
                _ => panic!("Image view is not supported (yet).")
            };

            if let Some(full_path) = gltf_dir_path.join(image_path).to_str() {
                material.albedo_map = Some(
                    Rc::clone(texture_manager.get_texture(full_path))
                )
            } else {
                log::warn!("Failed to get valid texture path for {}", image_path);
            }
        }

        material
    }
}
