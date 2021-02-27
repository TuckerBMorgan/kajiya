use anyhow::Result;

use memmap2::MmapOptions;
use std::fs::File;
use vicki::asset::mesh::{FlatImage, PackedTriMesh};

fn main() -> Result<()> {
    {
        let file = File::open("baked/derp.mesh")?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        let data: &[u8] = &mmap;
        let mesh: &PackedTriMesh::Flat =
            unsafe { (data.as_ptr() as *const PackedTriMesh::Flat).as_ref() }.unwrap();

        dbg!(mesh.verts.len());
        dbg!(mesh.uvs.len());
        dbg!(mesh.tangents.len());
    }

    {
        let file = File::open("baked/derp.image")?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        let data: &[u8] = &mmap;
        let image: &FlatImage::Flat =
            unsafe { (data.as_ptr() as *const FlatImage::Flat).as_ref() }.unwrap();

        dbg!(image.format);
        dbg!(image.extent);
        dbg!(image.mips.len());
        dbg!(image.mips[0].as_slice());
        dbg!(image.mips[1].as_slice());
        dbg!(image.mips[2].as_slice());
    }

    Ok(())
}