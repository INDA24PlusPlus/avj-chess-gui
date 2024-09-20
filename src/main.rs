use std::{env, path, time::Duration};

use ggez::*;
use glam::Vec2;
use graphics::{Color, MeshBuilder};

struct State {
    //    image: graphics::Image,
    rect: graphics::Mesh,
}

fn draw_board(mb: &mut MeshBuilder) {
    let white_square_color = graphics::Color::new(1.0, 1.0, 1.0, 1.0);
    let black_square_color = graphics::Color::new(0.0, 0.0, 0.0, 1.0);
    for i in 0..8 {
        for j in 0..8 {
            mb.rectangle(
                graphics::DrawMode::fill(),
                graphics::Rect::new(
                    100.0 + (j as f32 * 50.0),
                    100.0 + (i as f32 * 50.0),
                    50.0,
                    50.0,
                ),
                if (i + j) % 2 == 1 {
                    black_square_color
                } else {
                    white_square_color
                },
            )
            .unwrap();
        }
    }
}

impl State {
    fn new(ctx: &mut Context) -> GameResult<State> {
        //let image = graphics::Image::from_path(ctx, "/white_square.png")?;

        let mb = &mut graphics::MeshBuilder::new();
        mb.rectangle(
            graphics::DrawMode::fill(),
            graphics::Rect::new(450.0, 450.0, 50.0, 50.0),
            graphics::Color::new(1.0, 1.0, 1.0, 1.0),
        )?;

        draw_board(mb);

        let rect = graphics::Mesh::from_data(ctx, mb.build());
        let s = State { rect };

        Ok(s)
    }
}

impl event::EventHandler<ggez::GameError> for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const DESIRED_FPS: u32 = 60;

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));

        // Draw an image.
        let dst = glam::Vec2::new(20.0, 20.0);
        //canvas.draw(&self.image, graphics::DrawParam::new().dest(dst));

        // Draw an image with some options, and different filter modes.

        canvas.set_sampler(graphics::Sampler::nearest_clamp());
        canvas.set_default_sampler();

        // Draw a filled rectangle mesh.
        /* let rect = graphics::Rect::new(450.0, 450.0, 50.0, 50.0);
        canvas.draw(
            &graphics::Quad,
            graphics::DrawParam::new()
                .dest(rect.point())
                .scale(rect.size())
                .color(Color::WHITE),
        ); */

        // Draw a stroked rectangle mesh.
        canvas.draw(&self.rect, graphics::DrawParam::default());

        // Draw some pre-made meshes

        // Finished drawing, show it all on the screen!
        canvas.finish(ctx)?;

        Ok(())
    }
}

fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };
    let cb = ggez::ContextBuilder::new("drawing", "ggez").add_resource_path(resource_dir);

    let (mut ctx, events_loop) = cb.build()?;

    let state = State::new(&mut ctx).unwrap();
    event::run(ctx, events_loop, state)
}
