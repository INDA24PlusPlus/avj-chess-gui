use std::{collections::HashMap, env, fmt::Debug, path, time::Duration};

use conf::WindowMode;
use dexterws_chess::game::{
    Board, Color as PieceColor, GameResult as ChessResult, Move, Piece, Square,
};
use event::MouseButton;
use ggez::*;
use glam::Vec2;
use graphics::{Color, DrawParam, Drawable, FillOptions, MeshBuilder, Text};
use mint::Point2;

struct State {
    //    image: graphics::Image,
    rect: graphics::Mesh,
    board: Board,
    piece_images: [Option<graphics::Image>; 64],
    mouse_down: bool,
    current_legal_moves: Option<Vec<Move>>,
    selected_square: Option<Square>,
    past_moves: Vec<(PieceColor, Move)>,
}

fn piece_to_image(piece: (Piece, PieceColor)) -> String {
    let mut piece_image_map: HashMap<(Piece, PieceColor), String> = HashMap::new();
    piece_image_map.insert(
        (Piece::Bishop, PieceColor::White),
        String::from("/white_bishop.png"),
    );
    piece_image_map.insert(
        (Piece::Bishop, PieceColor::Black),
        String::from("/black_bishop.png"),
    );
    piece_image_map.insert(
        (Piece::Rook, PieceColor::White),
        String::from("/white_rook.png"),
    );
    piece_image_map.insert(
        (Piece::Rook, PieceColor::Black),
        String::from("/black_rook.png"),
    );
    piece_image_map.insert(
        (Piece::Pawn, PieceColor::White),
        String::from("/white_pawn.png"),
    );
    piece_image_map.insert(
        (Piece::Pawn, PieceColor::Black),
        String::from("/black_pawn.png"),
    );
    piece_image_map.insert(
        (Piece::Queen, PieceColor::White),
        String::from("/white_queen.png"),
    );
    piece_image_map.insert(
        (Piece::Queen, PieceColor::Black),
        String::from("/black_queen.png"),
    );
    piece_image_map.insert(
        (Piece::King, PieceColor::White),
        String::from("/white_king.png"),
    );
    piece_image_map.insert(
        (Piece::King, PieceColor::Black),
        String::from("/black_king.png"),
    );
    piece_image_map.insert(
        (Piece::Knight, PieceColor::White),
        String::from("/white_knight.png"),
    );
    piece_image_map.insert(
        (Piece::Knight, PieceColor::Black),
        String::from("/black_knight.png"),
    );

    return piece_image_map.get(&piece).unwrap().clone();
}

fn draw_board(mb: &mut MeshBuilder) {
    let white_square_color = graphics::Color::new(0.94, 0.85, 0.71, 1.0);
    let black_square_color = graphics::Color::new(0.71, 0.53, 0.39, 1.0);
    for i in 0..8 {
        for j in 0..8 {
            mb.rectangle(
                graphics::DrawMode::fill(),
                graphics::Rect::new(
                    100.0 + (j as f32 * 80.0),
                    100.0 + (i as f32 * 80.0),
                    80.0,
                    80.0,
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
        let board = Board::new();

        let mb = &mut graphics::MeshBuilder::new();

        draw_board(mb);

        let pieces = board.get_all_pieces();
        let piece_images: [Option<graphics::Image>; 64] = pieces.map(|piece| match piece {
            Some(p) => Some(graphics::Image::from_path(ctx, piece_to_image(p)).unwrap()),
            None => None,
        });

        let rect = graphics::Mesh::from_data(ctx, mb.build());
        let s = State {
            rect,
            board,
            piece_images,
            mouse_down: false,
            current_legal_moves: None,
            selected_square: None,
            past_moves: vec![],
        };

        Ok(s)
    }
}

impl event::EventHandler<ggez::GameError> for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const DESIRED_FPS: u32 = 60;

        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        self.mouse_down = true;

        if x < 740.0 && y < 740.0 && x > 100.0 && y > 100.0 {
            let file: u8 = ((x - 110.0) / 80.0) as u8;
            let rank: u8 = ((y - 110.0) / 80.0) as u8;
            println!("{}, {}, x: {}, y: {}", file, rank, x, y);
            let index = (rank * 8) + file;

            let square = Square::from_idx(index);

            if self.current_legal_moves.is_some()
                && self
                    .current_legal_moves
                    .as_ref()
                    .unwrap()
                    .iter()
                    .find(|m| m.to().file.to_idx() == file && m.to().rank.to_idx() == rank)
                    .is_some()
            {
                let selected_move = self
                    .current_legal_moves
                    .as_ref()
                    .unwrap()
                    .iter()
                    .find(|m| m.to().file.to_idx() == file && m.to().rank.to_idx() == rank)
                    .unwrap();

                self.board.make_move(*selected_move).unwrap();
                self.past_moves
                    .insert(0, (self.board.side(), *selected_move));
                let pieces: [Option<(Piece, PieceColor)>; 64] = self.board.get_all_pieces();
                let piece_images: [Option<graphics::Image>; 64] = pieces.map(|piece| match piece {
                    Some(p) => Some(graphics::Image::from_path(ctx, piece_to_image(p)).unwrap()),
                    None => None,
                });
                self.piece_images = piece_images;
                self.current_legal_moves = None;
            } else {
                // We are inside the board
                let legal_moves = self.board.get_moves(square);
                self.current_legal_moves = legal_moves;
                self.selected_square = Some(square);
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));
        let color = if self.board.side() == PieceColor::Black {
            "Black"
        } else {
            "White"
        };

        // Draw an image.
        //canvas.draw(&self.image, graphics::DrawParam::new().dest(dst));

        let status_text = match self.board.get_game_result() {
            ChessResult::Checkmate {
                winner: PieceColor::White,
            } => Text::new("Black in checkmate, white has won"),
            ChessResult::Checkmate {
                winner: PieceColor::Black,
            } => Text::new("White in checkmate, black has won"),
            ChessResult::Draw => Text::new("Game has been drawed"),
            ChessResult::FiftyMoveRule => Text::new("Game has been drawed due to 50 move rule"),
            ChessResult::ThreefoldRepetition => {
                Text::new("Game has been drawed due to three fold repition")
            }
            ChessResult::Stalemate => Text::new("Game has been drawed due to stalemate "),
            ChessResult::InProgress => Text::new("Game in progress"),
        };

        status_text.draw(
            &mut canvas,
            graphics::DrawParam::new().dest(glam::Vec2::new(400.0, 40.0)),
        );

        let side_text = Text::new(String::from(color) + " to move");
        side_text.draw(
            &mut canvas,
            graphics::DrawParam::new().dest(glam::Vec2::new(400.0, 70.0)),
        );

        for (index, piece_move) in self.past_moves.iter().enumerate() {
            let text_position = glam::Vec2::new(900.0, 110.0 + ((index as f32) * 40.0));
            let circle_position = glam::Vec2::new(880.0, 110.0 + ((index as f32) * 40.0));
            let circle_color = if piece_move.0 == PieceColor::Black {
                Color::new(1.0, 1.0, 1.0, 1.0)
            } else {
                Color::new(0.0, 0.0, 0.0, 1.0)
            };

            let color_circle = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::Fill(FillOptions::default()),
                circle_position,
                10.0,
                1.0,
                circle_color,
            )?;

            let text = Text::new(piece_move.1.to_string());

            color_circle.draw(&mut canvas, graphics::DrawParam::default());
            text.draw(
                &mut canvas,
                graphics::DrawParam::default().dest(text_position),
            );
        }

        canvas.set_sampler(graphics::Sampler::nearest_clamp());
        for (index, image) in self.piece_images.iter().enumerate() {
            if image.is_some() {
                let square = Square::from_idx(index as u8);
                let x_pos: f32 = 110.0 + (80.0 * (square.file.to_idx() as f32));
                let y_pos: f32 = 110.0 + (80.0 * (square.rank.to_idx() as f32));
                let image_destination = glam::Vec2::new(x_pos, y_pos);
                let piece_image = image.as_ref().unwrap();

                canvas.draw(
                    piece_image,
                    graphics::DrawParam::new().dest(image_destination).z(100),
                );
            }
        }
        if self.current_legal_moves.is_some() {
            for legal_move in self.current_legal_moves.as_ref().unwrap() {
                let new_position = Vec2::new(
                    140.0 + (80.0 * (legal_move.to().file.to_idx() as f32)),
                    140.0 + (80.0 * (legal_move.to().rank.to_idx() as f32)),
                );

                let circle = graphics::Mesh::new_circle(
                    ctx,
                    graphics::DrawMode::Fill(FillOptions::default()),
                    new_position,
                    10.0,
                    1.0,
                    Color::new(0.658, 0.654, 0.639, 1.0),
                )?;
                canvas.draw(&circle, graphics::DrawParam::default().z(99));
            }
        }
        // Draw an image with some options, and different filter modes.

        canvas.set_default_sampler();

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
    let window_mode = WindowMode::default().dimensions(1200.0, 1200.0);
    let cb = ggez::ContextBuilder::new("drawing", "ggez")
        .add_resource_path(resource_dir)
        .window_mode(window_mode.resizable(true));

    let (mut ctx, events_loop) = cb.build()?;

    let state = State::new(&mut ctx).unwrap();
    event::run(ctx, events_loop, state)
}
