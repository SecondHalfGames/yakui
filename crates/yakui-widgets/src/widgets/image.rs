use yakui_core::geometry::{Color, Constraints, Rect, Vec2};
use yakui_core::paint::PaintRect;
use yakui_core::widget::{LayoutContext, PaintContext, Widget};
use yakui_core::{Response, TextureId};

use crate::auto_builders;
use crate::util::widget;

/**
Displays an image.

Responds with [ImageResponse].
*/
#[derive(Debug, Clone)]
#[must_use = "yakui widgets do nothing if you don't `show` them"]
pub struct Image {
    pub image: Option<TextureId>,
    pub size: Vec2,
    pub color: Color,
    pub fit_mode: ImageFit,
}

auto_builders!(Image {
    size: Vec2,
    color: Color,
    fit_mode: ImageFit,
});

#[derive(Debug, Clone, Copy)]
pub enum ImageFit {
    Stretch,
    Fit,
}

impl Image {
    pub fn new<I>(image: I, size: Vec2) -> Self
    where
        I: Into<TextureId>,
    {
        Self {
            image: Some(image.into()),
            size,
            color: Color::WHITE,
            fit_mode: ImageFit::Fit,
        }
    }

    #[track_caller]
    pub fn show(self) -> Response<ImageResponse> {
        widget::<ImageWidget>(self)
    }
}

#[derive(Debug)]
pub struct ImageWidget;

pub type ImageResponse = ();

impl Widget for ImageWidget {
    type Props = Image;
    type Response = ImageResponse;

    fn new(_props: &Self::Props) -> Self {
        Self
    }

    fn update(&mut self, _props: &Self::Props) -> Self::Response {}

    fn layout(&self, props: &Self::Props, ctx: LayoutContext<'_>, input: Constraints) -> Vec2 {
        let mut output_size = input.constrain(props.size);

        match props.fit_mode {
            ImageFit::Stretch => {}

            ImageFit::Fit => {
                if let Some(TextureId::Managed(id)) = props.image {
                    let textures = ctx.paint.textures();
                    if let Some(texture) = textures.get(id) {
                        let real_size = texture.size().as_vec2();
                        let aspect_ratio = real_size.x / real_size.y;

                        let width_from_height = output_size.y * aspect_ratio;
                        let height_from_width = output_size.x / aspect_ratio;

                        if output_size.x < width_from_height {
                            output_size = Vec2::new(output_size.x, height_from_width);
                        } else {
                            output_size = Vec2::new(width_from_height, output_size.y);
                        }
                    }
                }
            }
        }

        output_size
    }

    fn paint(&self, props: &Self::Props, ctx: PaintContext<'_>) {
        let layout_node = ctx.layout.get(ctx.dom.current()).unwrap();

        if let Some(image) = props.image {
            let mut rect = PaintRect::new(layout_node.rect);
            rect.color = props.color;
            rect.texture = Some((image, Rect::ONE));
            rect.add(ctx.paint);
        }
    }
}
