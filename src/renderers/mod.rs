use std::cell::{Ref, RefCell};

use kajiya_backend::{
    rg::{self, GetOrCreateTemporal},
    Image,
};

pub mod csgi2;
pub mod half_res;
pub mod reprojection;
pub mod rtdgi;
pub mod rtr;
pub mod sky;
pub mod ssgi;
pub mod surfel_gi;
pub mod taa;

pub struct GbufferDepth {
    pub gbuffer: rg::Handle<Image>,
    pub depth: rg::Handle<Image>,
    half_view_normal: RefCell<Option<rg::Handle<Image>>>,
    half_depth: RefCell<Option<rg::Handle<Image>>>,
}

impl GbufferDepth {
    pub fn new(gbuffer: rg::Handle<Image>, depth: rg::Handle<Image>) -> Self {
        Self {
            gbuffer,
            depth,
            half_view_normal: Default::default(),
            half_depth: Default::default(),
        }
    }

    pub fn half_view_normal(&self, rg: &mut rg::RenderGraph) -> Ref<rg::Handle<Image>> {
        if self.half_view_normal.borrow().is_none() {
            *self.half_view_normal.borrow_mut() = Some(
                half_res::extract_half_res_gbuffer_view_normal_rgba8(rg, &self.gbuffer),
            );
        }

        Ref::map(self.half_view_normal.borrow(), |res| res.as_ref().unwrap())
    }

    pub fn half_depth(&self, rg: &mut rg::RenderGraph) -> Ref<rg::Handle<Image>> {
        if self.half_depth.borrow().is_none() {
            *self.half_depth.borrow_mut() = Some(half_res::extract_half_res_depth(rg, &self.depth));
        }

        Ref::map(self.half_depth.borrow(), |res| res.as_ref().unwrap())
    }
}

pub struct PingPongTemporalResource {
    pub output_tex: rg::TemporalResourceKey,
    pub history_tex: rg::TemporalResourceKey,
}

impl PingPongTemporalResource {
    pub fn new(name: &str) -> Self {
        Self {
            output_tex: format!("{}:0", name).as_str().into(),
            history_tex: format!("{}:1", name).as_str().into(),
        }
    }

    pub fn get_output_and_history(
        &mut self,
        rg: &mut rg::TemporalRenderGraph,
        desc: kajiya_backend::ImageDesc,
    ) -> (rg::Handle<Image>, rg::Handle<Image>) {
        let output_tex = rg
            .get_or_create_temporal(self.output_tex.clone(), desc)
            .unwrap();

        let history_tex = rg
            .get_or_create_temporal(self.history_tex.clone(), desc)
            .unwrap();

        std::mem::swap(&mut self.output_tex, &mut self.history_tex);

        (output_tex, history_tex)
    }
}
