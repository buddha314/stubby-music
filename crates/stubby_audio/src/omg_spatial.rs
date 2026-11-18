//! Experimental OMG Audio renderer used by the opt-in grotto review path.
//!
//! Geometry tracing and frame creation happen on the engine control thread. The
//! Firewheel processor owns a preconfigured renderer and allocates nothing while
//! rendering. Dynamic frame replacement is intentionally not implemented here.

use crate::protocol::{AcousticSceneGeometry, ListenerPose};
use firewheel::{
    channel_config::{ChannelConfig, ChannelCount},
    node::{
        AudioNode, AudioNodeInfo, AudioNodeProcessor, ConstructProcessorContext, EmptyConfig,
        NodeError, ProcBuffers, ProcExtra, ProcInfo, ProcessStatus,
    },
};
use omg_audio::{Config, Frame, Geometry, Material, Simulation, SourceDesc, SourceId, from_y_up};
use omg_core::params::ParamBlock;
use omg_dsp::{
    Renderer,
    ambi::NCH,
    output::OutputStage,
    renderer::{BLOCK, BusBlock},
};

/// Trace one static listener/source snapshot for the developer review harness.
/// This is deliberately synchronous and must not be called during ordinary
/// frame updates; the caller uses it only when the review camera changes view.
pub fn trace_snapshot(
    scene: &AcousticSceneGeometry,
    listener: ListenerPose,
    source_position: [f32; 3],
) -> Result<(Frame, SourceId), &'static str> {
    if scene.material_indices.len() != scene.triangles.len() {
        return Err("acoustic triangle/material count mismatch");
    }
    let mut geometry = Geometry::new();
    let materials: Vec<_> = scene
        .materials
        .iter()
        .map(|material| {
            geometry.material(Material {
                absorption: material.absorption,
                scattering: material.scattering,
                transmission: material.transmission,
            })
        })
        .collect();
    if scene.triangles.iter().any(|triangle| {
        triangle
            .iter()
            .any(|&index| index as usize >= scene.vertices.len())
    }) {
        return Err("acoustic triangle has an invalid vertex index");
    }
    let y_up_vertices: Vec<_> = scene
        .vertices
        .iter()
        .map(|&point| from_y_up(point))
        .collect();
    let indices: Vec<_> = scene
        .triangles
        .iter()
        .flat_map(|triangle| triangle.iter().copied())
        .collect();
    let triangle_materials: Option<Vec<_>> = scene
        .material_indices
        .iter()
        .map(|&index| materials.get(index).copied())
        .collect();
    let triangle_materials =
        triangle_materials.ok_or("acoustic triangle references missing material")?;
    geometry.add_mesh_per_tri(&y_up_vertices, &indices, &triangle_materials);

    // The current Firewheel adapter consumes ParamBlock taps/reverb directly;
    // OMG v4 additionally requires dense kernels from Frame::kernels, which this
    // one-source snapshot adapter does not install. Match the verified spike's
    // v6 path until the adapter carries those kernels as well.
    let mut simulation = Simulation::new(Config {
        v4: false,
        v5: false,
        v6: true,
        v61: true,
        v62: true,
        ..Config::default()
    });
    simulation.set_geometry(geometry);
    let listener_position = from_y_up(listener.position);
    let ahead = from_y_up(listener.ahead);
    simulation.set_listener(
        listener_position,
        ahead[1].atan2(ahead[0]),
        ahead[2].clamp(-1.0, 1.0).asin(),
        0.0,
    );
    let source = simulation.add_source(SourceDesc::at(from_y_up(source_position)));
    Ok((simulation.tick(), source))
}

#[derive(Clone)]
pub struct OmgSpatialNode {
    params: ParamBlock,
    gain: f32,
}

impl OmgSpatialNode {
    pub fn from_frame(frame: &Frame, source: SourceId, gain: f32) -> Option<Self> {
        Some(Self {
            params: frame.block(source)?.clone(),
            gain: gain.clamp(0.0, 1.0),
        })
    }
}

impl AudioNode for OmgSpatialNode {
    type Configuration = EmptyConfig;

    fn info(&self, _: &EmptyConfig) -> Result<AudioNodeInfo, NodeError> {
        Ok(AudioNodeInfo::new()
            .debug_name("omg-spatial-review")
            .channel_config(ChannelConfig::new(ChannelCount::MONO, ChannelCount::STEREO)))
    }

    fn construct_processor(
        &self,
        _: &EmptyConfig,
        cx: ConstructProcessorContext,
    ) -> Result<impl AudioNodeProcessor, NodeError> {
        let sample_rate = cx.stream_info.sample_rate.get() as f32;
        let mut renderer = Renderer::new(sample_rate);
        renderer.set_params(&self.params);
        Ok(OmgSpatialProcessor {
            renderer,
            gain: self.gain,
            output: OutputStage::from_speaker_bytes(None, sample_rate),
            bus: [[0.0; BLOCK]; NCH],
            point_left: [0.0; BLOCK],
            point_right: [0.0; BLOCK],
        })
    }
}

struct OmgSpatialProcessor {
    renderer: Renderer,
    gain: f32,
    output: OutputStage,
    bus: BusBlock,
    point_left: [f32; BLOCK],
    point_right: [f32; BLOCK],
}

impl AudioNodeProcessor for OmgSpatialProcessor {
    fn process(
        &mut self,
        info: &ProcInfo,
        ProcBuffers { inputs, outputs }: ProcBuffers,
        _: &mut ProcExtra,
    ) -> ProcessStatus {
        let (left_output, right_outputs) = outputs.split_at_mut(1);
        let (Some(input), Some(left), Some(right)) = (
            inputs.first(),
            left_output.first_mut(),
            right_outputs.first_mut(),
        ) else {
            return ProcessStatus::ClearAllOutputs;
        };
        let frames = info
            .frames
            .min(input.len())
            .min(left.len())
            .min(right.len());
        for start in (0..frames).step_by(BLOCK) {
            let count = (frames - start).min(BLOCK);
            for channel in &mut self.bus {
                channel[..count].fill(0.0);
            }
            self.point_left[..count].fill(0.0);
            self.point_right[..count].fill(0.0);
            self.renderer.process_block(
                &input[start..start + count],
                &mut self.bus,
                &mut self.point_left[..count],
                &mut self.point_right[..count],
            );
            for index in 0..count {
                let sh = std::array::from_fn(|channel| self.bus[channel][index]);
                let (l, r) =
                    self.output
                        .process(&sh, self.point_left[index], self.point_right[index]);
                left[start + index] = l * self.gain;
                right[start + index] = r * self.gain;
            }
        }
        ProcessStatus::OutputsModified
    }
}
