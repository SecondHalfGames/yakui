//! This example shows how you might draw a radial menu for a game using yakui
//! and lyon_tessellation.
//!
//! It also serves as an example showing how easy it is to use both libraries
//! together to draw interesting geometry!

use std::f32::consts::TAU;

use lyon_tessellation::geometry_builder::simple_builder;
use lyon_tessellation::math::{point, Point};
use lyon_tessellation::path::Path;
use lyon_tessellation::{
    FillOptions, FillTessellator, LineJoin, StrokeOptions, StrokeTessellator, VertexBuffers,
};
use yakui::{Color, Vec2};
use yakui_core::paint::{PaintMesh, Vertex};
use yakui_core::widget::PaintContext;
use yakui_widgets::widgets::ColoredBox;
use yakui_widgets::{canvas, center};

pub fn run() {
    center(|| {
        ColoredBox::sized(Color::BLACK, Vec2::splat(500.0)).show_children(|| {
            center(|| {
                canvas(|ctx| {
                    let num_segments = 3;
                    let gap = TAU / 64.0;

                    for i in 0..num_segments {
                        let layout_node = ctx.layout.get(ctx.dom.current()).unwrap();
                        let center = layout_node.rect.pos() + layout_node.rect.size() / 2.0;

                        let frac = (i as f32) / (num_segments as f32);
                        let start_angle = frac * TAU + (gap / 2.0);
                        let end_angle = start_angle + TAU / (num_segments as f32) - gap;

                        draw_segment(ctx, center, start_angle, end_angle);
                    }
                });
            });
        });
    });
}

fn draw_segment(ctx: &mut PaintContext<'_>, center: Vec2, start_angle: f32, end_angle: f32) {
    let inner_radius = 100.0;
    let outer_radius = 150.0;
    let angle_samples = 16;

    let angle_range = (end_angle - start_angle).abs();

    let mut builder = Path::builder();
    builder.begin(point(start_angle.cos(), start_angle.sin()) * inner_radius);
    builder.line_to(point(start_angle.cos(), start_angle.sin()) * outer_radius);

    for i in 0..=angle_samples {
        let fraction = (i as f32) / (angle_samples as f32);
        let angle = start_angle + angle_range * fraction;
        builder.line_to(point(angle.cos(), angle.sin()) * outer_radius);
    }

    builder.line_to(point(end_angle.cos(), end_angle.sin()) * inner_radius);

    for i in (0..=angle_samples).rev() {
        let fraction = (i as f32) / (angle_samples as f32);
        let angle = start_angle + angle_range * fraction;
        builder.line_to(point(angle.cos(), angle.sin()) * inner_radius);
    }

    builder.end(true);

    let path = builder.build();

    {
        let mut buffers: VertexBuffers<Point, u16> = VertexBuffers::new();
        let mut vertex_builder = simple_builder(&mut buffers);
        let mut tessellator = FillTessellator::new();

        tessellator
            .tessellate_path(&path, &FillOptions::default(), &mut vertex_builder)
            .unwrap();

        let vertices = buffers.vertices.iter().map(|point| {
            let pos = center + Vec2::from(point.to_array());
            Vertex::new(pos, pos, Color::hex(0x333333).to_linear())
        });

        let mesh = PaintMesh::new(vertices, buffers.indices);
        ctx.paint.add_mesh(mesh);
    }

    {
        let mut buffers: VertexBuffers<Point, u16> = VertexBuffers::new();
        let mut vertex_builder = simple_builder(&mut buffers);
        let mut tessellator = StrokeTessellator::new();

        let options = StrokeOptions::default()
            .with_line_width(2.0)
            .with_line_join(LineJoin::Round);

        tessellator
            .tessellate(&path, &options, &mut vertex_builder)
            .unwrap();

        let vertices = buffers.vertices.iter().map(|point| {
            let pos = center + Vec2::from(point.to_array());
            Vertex::new(pos, pos, Color::WHITE.to_linear())
        });

        let mesh = PaintMesh::new(vertices, buffers.indices);
        ctx.paint.add_mesh(mesh);
    }
}

fn main() {
    bootstrap::start(run as fn());
}
