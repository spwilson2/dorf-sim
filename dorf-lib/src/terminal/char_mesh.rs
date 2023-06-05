use crate::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct CharMesh {
    pub texture_vec: Vec<CharTexture>,
}

#[derive(Bundle, Clone, Debug)]
pub struct CharMeshTransform {
    mesh: CharMesh,
    transform: Transform2D,
}

impl CharMeshTransform {
    #[inline]
    pub fn new(transform: Transform2D) -> Self {
        Self {
            mesh: CharMesh {
                texture_vec: vec![default(); (transform.scale.x * transform.scale.y) as usize],
            },
            transform,
        }
    }

    #[inline]
    pub fn transform(&self) -> &Transform2D {
        &self.transform
    }

    #[inline]
    pub fn from_parts(mesh: CharMesh, transform: Transform2D) -> Self {
        Self { mesh, transform }
    }

    /// Return a refernce to the [`CharTexture`] using global coordinates.
    /// Panics if the mesh does not cover the provided coordinates.
    pub fn get(&self, x: i32, y: i32) -> &CharTexture {
        // TODO: Release version with unsafe unwrap
        return self
            .mesh
            .texture_vec
            .get(
                self.transform
                    .as_rect2d()
                    .index_for_point(IVec2::new(x, y))
                    .unwrap(),
            )
            .unwrap();
    }

    /// Mutable verison of [`get`].
    pub fn get_mut(&mut self, x: i32, y: i32) -> &mut CharTexture {
        // TODO: Release version with unsafe unwrap
        return self
            .mesh
            .texture_vec
            .get_mut(
                self.transform
                    .as_rect2d()
                    .index_for_point(IVec2::new(x, y))
                    .unwrap(),
            )
            .unwrap();
    }

    /// Return a reference to the given texture using coordinates local to the
    /// Mesh (i.e. 0,0 is the topleft of the mesh, no matter where its transform
    /// is located.)
    pub fn get_local(&self, x: usize, y: usize) -> &CharTexture {
        // TODO: Release version with unsafe unwrap
        let width = self.transform.scale.x;
        return self.mesh.texture_vec.get(width as usize * x + y).unwrap();
    }

    pub fn fill(&mut self, texture: &CharTexture) {
        for point in self.mesh.texture_vec.iter_mut() {
            *point = texture.clone()
        }
    }
    /// Mutable version of [`get_local`].
    pub fn get_local_mut(&mut self, x: usize, y: usize) -> &mut CharTexture {
        // TODO: Release version with unsafe unwrap
        let width = self.transform.scale.x;
        return self
            .mesh
            .texture_vec
            .get_mut(width as usize * x + y)
            .unwrap();
    }
}
