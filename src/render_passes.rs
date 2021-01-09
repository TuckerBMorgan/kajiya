use std::sync::Arc;

use slingshot::ash::{version::DeviceV1_0, vk};

use crate::{backend::image::ImageViewDesc, backend::shader::*, rg::*};

pub fn create_image(rg: &mut RenderGraph, desc: ImageDesc) -> Handle<Image> {
    let mut pass = rg.add_pass();
    pass.create(&desc)
}

pub fn clear_depth(rg: &mut RenderGraph, img: &mut Handle<Image>) {
    let mut pass = rg.add_pass();
    let output_ref = pass.write(img, AccessType::TransferWrite);

    pass.render(move |api| {
        let raw_device = &api.device().raw;
        let cb = api.cb;

        let image = api.resources.image(output_ref);

        unsafe {
            raw_device.cmd_clear_depth_stencil_image(
                cb.raw,
                image.raw,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &vk::ClearDepthStencilValue {
                    depth: 0f32,
                    stencil: 0,
                },
                std::slice::from_ref(&vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL,
                    level_count: 1 as u32,
                    layer_count: 1,
                    ..Default::default()
                }),
            );
        }
    });
}

pub fn clear_color(rg: &mut RenderGraph, img: &mut Handle<Image>, clear_color: [f32; 4]) {
    let mut pass = rg.add_pass();
    let output_ref = pass.write(img, AccessType::TransferWrite);

    pass.render(move |api| {
        let raw_device = &api.device().raw;
        let cb = api.cb;

        let image = api.resources.image(output_ref);

        unsafe {
            raw_device.cmd_clear_color_image(
                cb.raw,
                image.raw,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &vk::ClearColorValue {
                    float32: clear_color,
                },
                std::slice::from_ref(&vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    level_count: 1 as u32,
                    layer_count: 1,
                    ..Default::default()
                }),
            );
        }
    });
}

/*pub fn raymarch_sdf(
    rg: &mut RenderGraph,
    sdf_img: &Handle<Image>,
    desc: ImageDesc,
) -> Handle<Image> {
    let mut pass = rg.add_pass();

    let pipeline = pass.register_compute_pipeline("/assets/shaders/sdf/sdf_raymarch_gbuffer.hlsl");

    let sdf_ref = pass.read(
        sdf_img,
        AccessType::ComputeShaderReadSampledImageOrUniformTexelBuffer,
    );
    let mut output = pass.create(&desc);
    let output_ref = pass.write(&mut output, AccessType::ComputeShaderWrite);

    pass.render(move |api| {
        let pipeline = api.bind_compute_pipeline(pipeline.into_binding().descriptor_set(
            0,
            &[
                output_ref.bind(ImageViewDescBuilder::default()),
                sdf_ref.bind(ImageViewDescBuilder::default()),
            ],
        ));

        pipeline.dispatch(desc.extent);
    });

    output
}

pub fn edit_sdf(rg: &mut RenderGraph, sdf_img: &mut Handle<Image>, clear: bool) {
    let mut pass = rg.add_pass();

    let sdf_img_ref = pass.write(sdf_img, AccessType::ComputeShaderWrite);

    let pipeline_path = if clear {
        "/assets/shaders/sdf/gen_empty_sdf.hlsl"
    } else {
        "/assets/shaders/sdf/edit_sdf.hlsl"
    };

    let pipeline = pass.register_compute_pipeline(pipeline_path);

    pass.render(move |api| {
        let pipeline = api.bind_compute_pipeline(
            pipeline
                .into_binding()
                .descriptor_set(0, &[sdf_img_ref.bind(ImageViewDescBuilder::default())]),
        );
        pipeline.dispatch([SDF_DIM, SDF_DIM, SDF_DIM]);
    });
}

fn clear_sdf_bricks_meta(rg: &mut RenderGraph) -> Handle<Buffer> {
    let mut pass = rg.add_pass();

    let mut brick_meta_buf = pass.create(&BufferDesc {
        size: 20, // size of VkDrawIndexedIndirectCommand
        usage: vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::INDIRECT_BUFFER,
    });
    let brick_meta_buf_ref = pass.write(&mut brick_meta_buf, AccessType::ComputeShaderWrite);

    let clear_meta_pipeline =
        pass.register_compute_pipeline("/assets/shaders/sdf/clear_bricks_meta.hlsl");

    pass.render(move |api| {
        let pipeline = api.bind_compute_pipeline(
            clear_meta_pipeline
                .into_binding()
                .descriptor_set(0, &[brick_meta_buf_ref.bind()]),
        );
        pipeline.dispatch([1, 1, 1]);
    });

    brick_meta_buf
}

pub struct SdfRasterBricks {
    pub brick_meta_buffer: Handle<Buffer>,
    pub brick_inst_buffer: Handle<Buffer>,
}

pub fn calculate_sdf_bricks_meta(rg: &mut RenderGraph, sdf_img: &Handle<Image>) -> SdfRasterBricks {
    let mut brick_meta_buf = clear_sdf_bricks_meta(rg);

    let mut pass = rg.add_pass();

    let sdf_ref = pass.read(
        sdf_img,
        AccessType::ComputeShaderReadSampledImageOrUniformTexelBuffer,
    );

    let brick_meta_buf_ref = pass.write(&mut brick_meta_buf, AccessType::ComputeShaderWrite);

    let mut brick_inst_buf = pass.create(&BufferDesc {
        size: (SDF_DIM as usize).pow(3) * 4 * 4,
        usage: vk::BufferUsageFlags::STORAGE_BUFFER,
    });
    let brick_inst_buf_ref = pass.write(&mut brick_inst_buf, AccessType::ComputeShaderWrite);

    let calc_meta_pipeline = pass.register_compute_pipeline("/assets/shaders/sdf/find_bricks.hlsl");

    pass.render(move |api| {
        let pipeline = api.bind_compute_pipeline(calc_meta_pipeline.into_binding().descriptor_set(
            0,
            &[
                sdf_ref.bind(ImageViewDescBuilder::default()),
                brick_meta_buf_ref.bind(),
                brick_inst_buf_ref.bind(),
            ],
        ));
        pipeline.dispatch([SDF_DIM / 2, SDF_DIM / 2, SDF_DIM / 2]);
    });

    SdfRasterBricks {
        brick_meta_buffer: brick_meta_buf,
        brick_inst_buffer: brick_inst_buf,
    }
}

pub struct RasterSdfData<'a> {
    pub sdf_img: &'a Handle<Image>,
    pub brick_inst_buffer: &'a Handle<Buffer>,
    pub brick_meta_buffer: &'a Handle<Buffer>,
    pub cube_index_buffer: &'a Handle<Buffer>,
}

pub fn raster_sdf(
    rg: &mut RenderGraph,
    render_pass: Arc<RenderPass>,
    depth_img: &mut Handle<Image>,
    color_img: &mut Handle<Image>,
    raster_sdf_data: RasterSdfData<'_>,
) {
    let mut pass = rg.add_pass();

    let pipeline = pass.register_raster_pipeline(
        &[
            RasterPipelineShader {
                code: "/assets/shaders/raster_simple_vs.hlsl",
                desc: RasterShaderDesc::builder(RasterStage::Vertex)
                    .build()
                    .unwrap(),
            },
            RasterPipelineShader {
                code: "/assets/shaders/raster_simple_ps.hlsl",
                desc: RasterShaderDesc::builder(RasterStage::Pixel)
                    .build()
                    .unwrap(),
            },
        ],
        RasterPipelineDesc::builder()
            .render_pass(render_pass.clone())
            .face_cull(true),
    );

    let sdf_ref = pass.read(
        raster_sdf_data.sdf_img,
        AccessType::FragmentShaderReadSampledImageOrUniformTexelBuffer,
    );
    let brick_inst_buffer = pass.read(
        raster_sdf_data.brick_inst_buffer,
        AccessType::VertexShaderReadSampledImageOrUniformTexelBuffer,
    );
    let brick_meta_buffer = pass.read(
        raster_sdf_data.brick_meta_buffer,
        AccessType::IndirectBuffer,
    );
    let cube_index_buffer = pass.read(raster_sdf_data.cube_index_buffer, AccessType::IndexBuffer);

    let depth_ref = pass.raster(depth_img, AccessType::DepthStencilAttachmentWrite);
    let color_ref = pass.raster(color_img, AccessType::ColorAttachmentWrite);

    pass.render(move |api| {
        let [width, height, _] = color_ref.desc().extent;

        api.begin_render_pass(
            &*render_pass,
            [width, height],
            &[(color_ref, &ImageViewDesc::default())],
            Some((
                depth_ref,
                &ImageViewDesc::builder()
                    .aspect_mask(vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL)
                    .build()
                    .unwrap(),
            )),
        );

        api.set_default_view_and_scissor([width, height]);

        let _pipeline = api.bind_raster_pipeline(pipeline.into_binding().descriptor_set(
            0,
            &[
                brick_inst_buffer.bind(),
                sdf_ref.bind(ImageViewDescBuilder::default()),
            ],
        ));

        unsafe {
            let raw_device = &api.device().raw;
            let cb = api.cb;

            raw_device.cmd_bind_index_buffer(
                cb.raw,
                api.resources.buffer(cube_index_buffer).raw,
                0,
                vk::IndexType::UINT32,
            );

            raw_device.cmd_draw_indexed_indirect(
                cb.raw,
                api.resources.buffer(brick_meta_buffer).raw,
                0,
                1,
                0,
            );
        }

        api.end_render_pass();
    });
}*/

#[derive(Clone)]
pub struct UploadedTriMesh {
    pub index_buffer: Arc<Buffer>,
    pub index_count: u32,
}

pub struct RasterMeshesData<'a> {
    pub meshes: &'a [UploadedTriMesh],
    pub mesh_buffer: &'a Handle<Buffer>,
    pub vertex_buffer: &'a Handle<Buffer>,
    pub bindless_descriptor_set: vk::DescriptorSet,
}

pub fn raster_meshes(
    rg: &mut RenderGraph,
    render_pass: Arc<RenderPass>,
    depth_img: &mut Handle<Image>,
    color_img: &mut Handle<Image>,
    mesh_data: RasterMeshesData<'_>,
) {
    let mut pass = rg.add_pass();

    let pipeline = pass.register_raster_pipeline(
        &[
            RasterPipelineShader {
                code: "/assets/shaders/raster_simple_vs.hlsl",
                desc: RasterShaderDesc::builder(RasterStage::Vertex)
                    .build()
                    .unwrap(),
            },
            RasterPipelineShader {
                code: "/assets/shaders/raster_simple_ps.hlsl",
                desc: RasterShaderDesc::builder(RasterStage::Pixel)
                    .build()
                    .unwrap(),
            },
        ],
        RasterPipelineDesc::builder()
            .render_pass(render_pass.clone())
            .face_cull(true),
    );

    let chunks: Vec<UploadedTriMesh> = mesh_data.meshes.to_owned();

    let depth_ref = pass.raster(depth_img, AccessType::DepthStencilAttachmentWrite);
    let color_ref = pass.raster(color_img, AccessType::ColorAttachmentWrite);

    let meshes_ref = pass.read(
        mesh_data.mesh_buffer,
        AccessType::VertexShaderReadSampledImageOrUniformTexelBuffer,
    );

    let vb_ref = pass.read(
        mesh_data.vertex_buffer,
        AccessType::VertexShaderReadSampledImageOrUniformTexelBuffer,
    );

    let bindless_descriptor_set = mesh_data.bindless_descriptor_set;

    pass.render(move |api| {
        let [width, height, _] = color_ref.desc().extent;

        api.begin_render_pass(
            &*render_pass,
            [width, height],
            &[(color_ref, &ImageViewDesc::default())],
            Some((
                depth_ref,
                &ImageViewDesc::builder()
                    .aspect_mask(vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL)
                    .build()
                    .unwrap(),
            )),
        );

        api.set_default_view_and_scissor([width, height]);

        let _pipeline = api.bind_raster_pipeline(
            pipeline
                .into_binding()
                .descriptor_set(1, &[meshes_ref.bind(), vb_ref.bind()])
                .raw_descriptor_set(3, bindless_descriptor_set),
        );

        unsafe {
            let raw_device = &api.device().raw;
            let cb = api.cb;

            for chunk in chunks {
                raw_device.cmd_bind_index_buffer(
                    cb.raw,
                    chunk.index_buffer.raw,
                    0,
                    vk::IndexType::UINT32,
                );

                raw_device.cmd_draw_indexed(cb.raw, chunk.index_count, 1, 0, 0, 0);
            }
        }

        api.end_render_pass();
    });
}

pub fn blur(rg: &mut RenderGraph, input: &Handle<Image>) -> Handle<Image> {
    let mut pass = rg.add_pass();

    let pipeline = pass.register_compute_pipeline("/assets/shaders/blur.hlsl");

    let input_ref = pass.read(
        input,
        AccessType::ComputeShaderReadSampledImageOrUniformTexelBuffer,
    );
    let mut output = pass.create(input.desc());
    let output_ref = pass.write(&mut output, AccessType::ComputeShaderWrite);

    pass.render(move |api| {
        let pipeline = api.bind_compute_pipeline(pipeline.into_binding().descriptor_set(
            0,
            &[
                input_ref.bind(ImageViewDescBuilder::default()),
                output_ref.bind(ImageViewDescBuilder::default()),
            ],
        ));

        pipeline.dispatch(input_ref.desc().extent);
    });

    output
}