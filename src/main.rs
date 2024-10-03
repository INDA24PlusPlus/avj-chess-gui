use std::io::{ErrorKind, Write};
use std::{
    collections::HashMap,
    env,
    io::Read,
    net::{SocketAddr, TcpListener, TcpStream},
    path,
};

use chess_networking::{Ack, GameState, Move as NetworkMove, PromotionPiece, Start};
use conf::WindowMode;
use dexterws_chess::game::{
    Board, Color as PieceColor, File, GameResult as ChessResult, Move, Piece, Rank, Square,
};
use event::MouseButton;
use ggez::*;
use glam::Vec2;
use graphics::{Color, Drawable, FillOptions, MeshBuilder, Text};

fn piece_to_promotion_piece(piece: Option<Piece>) -> Option<PromotionPiece> {
    match piece {
        Some(Piece::Queen) => Some(PromotionPiece::Queen),
        Some(Piece::Bishop) => Some(PromotionPiece::Bishop),
        Some(Piece::Knight) => Some(PromotionPiece::Knight),
        Some(Piece::Rook) => Some(PromotionPiece::Rook),
        _ => None,
    }
}

fn promotion_piece_to_piece(piece: Option<PromotionPiece>) -> Option<Piece> {
    match piece {
        Some(PromotionPiece::Queen) => Some(Piece::Queen),
        Some(PromotionPiece::Bishop) => Some(Piece::Bishop),
        Some(PromotionPiece::Knight) => Some(Piece::Knight),
        Some(PromotionPiece::Rook) => Some(Piece::Rook),
        _ => None,
    }
}

struct State {
    //    image: graphics::Image,
    rect: graphics::Mesh,
    board: Board,
    piece_images: [Option<graphics::Image>; 64],
    mouse_down: bool,
    current_legal_moves: Option<Vec<Move>>,
    selected_square: Option<Square>,
    past_moves: Vec<(PieceColor, Move)>,
    // None = not connected, true = host, false = join
    is_host: Option<bool>,
    selected_color: Option<PieceColor>,
    client_stream: Option<TcpStream>,
    start: Option<Start>,
    pending_chess_move: Option<Move>,
}

fn handle_client(mut stream: TcpStream, state: &mut State) {
    let mut buf = vec![];
    match stream.set_nonblocking(true) {
        Ok(_) => (),
        Err(e) => println!("Error setting nonblocking: {}", e),
    }
    loop {
        println!("Looping");
        match stream.read_to_end(&mut buf) {
            Ok(_) => {
                println!("Client start received");
                break;
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                println!("WouldBlock");
                continue;
            }
            Err(e) => panic!("encountered IO error: {e}"),
        };
    }

    match Start::try_from(&buf[..]) {
        Ok(start) => {
            println!("Host start: {:?}", start);
            let client_start = Start {
                is_white: state.selected_color.unwrap() == PieceColor::White,
                name: None,
                fen: None,
                time: None,
                inc: None,
            };
            let client_start_bytes: Vec<u8> = client_start.try_into().unwrap();
            println!("Host sending client start: {}", client_start_bytes.len());
            match stream.write(&client_start_bytes) {
                Ok(_) => println!("Client confirmed start sent"),
                Err(e) => println!("Error sending client start: {}", e),
            }
        }
        Err(e) => println!("Error host parsing start: {}", e),
    }
}

fn listen_for_connections(state: &mut State) {
    let addrs = [
        SocketAddr::from(([127, 0, 0, 1], 8080)),
        SocketAddr::from(([127, 0, 0, 1], 8081)),
    ];
    let listener: TcpListener = TcpListener::bind(&addrs[..]).unwrap();
    println!("Listening for connections");
    listener.set_nonblocking(true).unwrap();

    loop {
        match listener.accept() {
            Ok((stream, addr)) => {
                println!("New connection: {}", addr);
                stream
                    .set_nonblocking(true)
                    .expect("Failed to set client stream to non-blocking");
                state.client_stream = Some(stream);
                state.is_host = Some(true);
                break;
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No connection available, continue looping
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(e) => println!("Error accepting connection: {}", e),
        }
    }
    /* println!("Client accepted connection: {}", _addr);
    handle_client(stream, state); */
}

fn connect_to_host(address: String, state: &mut State) {
    match TcpStream::connect(address) {
        Ok(mut stream) => {
            println!("Connected to server: {}", stream.peer_addr().unwrap());
            stream
                .set_nonblocking(true)
                .expect("Failed to set non-blocking");
            state.client_stream = Some(stream);
            state.is_host = Some(false);
        }
        Err(e) => println!("Failed to connect: {}", e),
    }
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

fn draw_restart_button(canvas: &mut graphics::Canvas, ctx: &mut Context, is_host: Option<bool>) {
    let button_host = graphics::Mesh::new_rounded_rectangle(
        ctx,
        graphics::DrawMode::fill(),
        graphics::Rect::new(640.0, 800.0, 150.0, 40.0),
        5.0,
        graphics::Color::new(1.0, 0.0, 0.0, 1.0),
    )
    .unwrap();
    let button_join = graphics::Mesh::new_rounded_rectangle(
        ctx,
        graphics::DrawMode::fill(),
        graphics::Rect::new(640.0, 850.0, 150.0, 40.0),
        5.0,
        graphics::Color::new(1.0, 0.0, 0.0, 1.0),
    )
    .unwrap();

    let button_init_game = graphics::Mesh::new_rounded_rectangle(
        ctx,
        graphics::DrawMode::fill(),
        graphics::Rect::new(640.0, 900.0, 150.0, 40.0),
        5.0,
        graphics::Color::new(1.0, 0.0, 0.0, 1.0),
    )
    .unwrap();

    let button_text_host = Text::new("New game + (host)");
    let button_text_join = Text::new("New game + (join)");
    let button_text_init_game = Text::new("Init game");
    let text_position_host = glam::Vec2::new(650.0, 810.0);
    let text_position_join = glam::Vec2::new(650.0, 860.0);
    let text_position_init_game = glam::Vec2::new(650.0, 910.0);
    button_host.draw(canvas, graphics::DrawParam::default());
    button_join.draw(canvas, graphics::DrawParam::default());
    button_text_host.draw(
        canvas,
        graphics::DrawParam::default()
            .z(100)
            .dest(text_position_host),
    );
    button_text_join.draw(
        canvas,
        graphics::DrawParam::default()
            .z(100)
            .dest(text_position_join),
    );
    if is_host.is_some() && is_host.unwrap() == false {
        button_init_game.draw(canvas, graphics::DrawParam::default());
        button_text_init_game.draw(
            canvas,
            graphics::DrawParam::default()
                .z(100)
                .dest(text_position_init_game),
        );
    }
}

fn draw_color_picker(canvas: &mut graphics::Canvas, ctx: &mut Context) {
    let label_text = Text::new("Choose color");
    let label_position = glam::Vec2::new(500.0, 780.0);
    let white_rect = graphics::Mesh::new_rounded_rectangle(
        ctx,
        graphics::DrawMode::fill(),
        graphics::Rect::new(500.0, 800.0, 60.0, 40.0),
        5.0,
        graphics::Color::new(1.0, 1.0, 1.0, 1.0),
    )
    .unwrap();

    let black_rect = graphics::Mesh::new_rounded_rectangle(
        ctx,
        graphics::DrawMode::fill(),
        graphics::Rect::new(500.0, 850.0, 60.0, 40.0),
        5.0,
        graphics::Color::new(0.0, 0.0, 0.0, 1.0),
    )
    .unwrap();

    label_text.draw(canvas, graphics::DrawParam::default().dest(label_position));

    white_rect.draw(canvas, graphics::DrawParam::default());
    black_rect.draw(canvas, graphics::DrawParam::default());
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
            is_host: None,
            selected_color: None,
            client_stream: None,
            start: None,
            pending_chess_move: None,
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
            let piece = self.board.get_piece(square);

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

                /* if false {
                    self.board.make_move(*selected_move).unwrap();
                } */
                self.pending_chess_move = Some(*selected_move);
                let promotion_piece = match selected_move.promotion() {
                    Some(p) => Some(p),
                    None => None,
                };
                println!("Selected move: {:?}", selected_move);
                let network_move = NetworkMove {
                    from: (
                        selected_move.from().file.to_idx(),
                        selected_move.from().rank.to_idx(),
                    ),
                    to: (
                        selected_move.to().file.to_idx(),
                        selected_move.to().rank.to_idx(),
                    ),
                    forfeit: false,
                    offer_draw: false,
                    promotion: piece_to_promotion_piece(promotion_piece),
                };
                let network_move_bytes: Vec<u8> = network_move.try_into().unwrap();
                match self.client_stream.as_mut() {
                    Some(stream) => match stream.write_all(&network_move_bytes) {
                        Ok(_) => println!("Move sent successfully"),
                        Err(e) => println!("Error sending move: {}", e),
                    },
                    None => println!("No client stream found"),
                }
                /* if false {
                    self.past_moves
                        .insert(0, (self.board.side(), *selected_move));
                    let pieces: [Option<(Piece, PieceColor)>; 64] = self.board.get_all_pieces();
                    let piece_images: [Option<graphics::Image>; 64] =
                        pieces.map(|piece| match piece {
                            Some(p) => {
                                Some(graphics::Image::from_path(ctx, piece_to_image(p)).unwrap())
                            }
                            None => None,
                        });
                    self.piece_images = piece_images;
                } */
                self.current_legal_moves = None;
            } else if (piece.is_some_and(|p| p.0 == PieceColor::White)
                && self.start.as_ref().unwrap().is_white == true)
                || (piece.is_some_and(|p| p.0 == PieceColor::Black)
                    && self.start.as_ref().unwrap().is_white == false)
            {
                // We are inside the board
                println!("Selected square: {:?}", square);
                let legal_moves = self.board.get_moves(square);
                println!("Legal moves: {:?}", legal_moves);
                self.current_legal_moves = legal_moves;
                self.selected_square = Some(square);
            }
        }

        // restart button (host) has been pressed
        if x >= 640.0 && x <= 750.0 && y >= 800.0 && y <= 840.0 {
            self.current_legal_moves = Some(vec![]);
            self.past_moves = vec![];
            self.selected_square = None;
            self.board = Board::new();
            let piece_images: [Option<graphics::Image>; 64] =
                self.board.get_all_pieces().map(|piece| match piece {
                    Some(p) => Some(graphics::Image::from_path(ctx, piece_to_image(p)).unwrap()),
                    None => None,
                });
            self.piece_images = piece_images;
            listen_for_connections(self);
            self.is_host = Some(true);
        }

        // restart button (join) has been pressed
        if x >= 640.0 && x <= 750.0 && y >= 850.0 && y <= 890.0 {
            connect_to_host("127.0.0.1:8080".to_string(), self);
        }
        if x >= 500.0 && x <= 560.0 && y >= 800.0 && y <= 840.0 {
            self.selected_color = Some(PieceColor::White);
        }
        if x >= 500.0 && x <= 560.0 && y >= 850.0 && y <= 890.0 {
            self.selected_color = Some(PieceColor::Black);
        }
        if x >= 640.0
            && x <= 750.0
            && y >= 900.0
            && y <= 940.0
            && self.is_host.is_some()
            && self.is_host.unwrap() == false
        {
            println!("Init game button pressed");
            let start_package = Start {
                is_white: self.selected_color.unwrap() == PieceColor::White,
                name: None,
                fen: None,
                time: None,
                inc: None,
            };
            let start_package_bytes: Vec<u8> = start_package.try_into().unwrap();
            println!(
                "Peer address: {}",
                self.client_stream.as_ref().unwrap().peer_addr().unwrap()
            );
            if let Some(stream) = self.client_stream.as_mut() {
                match stream.write_all(&start_package_bytes) {
                    Ok(_) => {
                        println!(
                            "Init game package sent: {} bytes",
                            start_package_bytes.len()
                        );
                        match stream.flush() {
                            Ok(_) => println!("Stream flushed successfully"),
                            Err(e) => println!("Error flushing stream: {}", e),
                        }
                    }
                    Err(e) => println!("Error sending init game package: {}", e),
                }
            } else {
                println!("No client stream found");
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas: graphics::Canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));
        let color = if self.board.side() == PieceColor::Black {
            "Black"
        } else {
            "White"
        };
        draw_color_picker(&mut canvas, ctx);
        // Draw an image.
        //canvas.draw(&self.image, graphics::DrawParam::new().dest(dst));

        let status_text = match self.board.get_game_result() {
            ChessResult::Checkmate {
                winner: PieceColor::White,
            } => Text::new("Black in checkmate, white has won. Press restart to start new game."),
            ChessResult::Checkmate {
                winner: PieceColor::Black,
            } => Text::new("White in checkmate, black has won. Press restart to start new game"),
            ChessResult::Draw => {
                Text::new("Game has been drawed. Press restart to start new game.")
            }
            ChessResult::FiftyMoveRule => Text::new(
                "Game has been drawed due to 50 move rule. Press restart to start new game",
            ),
            ChessResult::ThreefoldRepetition => Text::new(
                "Game has been drawed due to three fold repition. Press restart to start new game",
            ),
            ChessResult::Stalemate => {
                Text::new("Game has been drawed due to stalemate. Press restart to start new game")
            }
            ChessResult::InProgress => Text::new("Game in progress"),
        };

        // display rank and file
        for i in 0..8 {
            // display rank
            let rank_text = Text::new((i + 1).to_string());
            rank_text.draw(
                &mut canvas,
                graphics::DrawParam::new().dest(glam::Vec2::new(80.0, 130.0 + ((i as f32) * 80.0))),
            );
            // display file
            let file_text = Text::new(((b'a' + i as u8) as char).to_string());
            file_text.draw(
                &mut canvas,
                graphics::DrawParam::new()
                    .dest(glam::Vec2::new(130.0 + ((i as f32) * 80.0), 750.0)),
            );
        }

        status_text.draw(
            &mut canvas,
            graphics::DrawParam::new().dest(glam::Vec2::new(400.0, 40.0)),
        );

        if self.board.get_game_result() == ChessResult::InProgress {
            let side_text = Text::new(String::from(color) + " to move");
            side_text.draw(
                &mut canvas,
                graphics::DrawParam::new().dest(glam::Vec2::new(400.0, 70.0)),
            );
        }

        // restart button
        if self.selected_color.is_some() {
            draw_restart_button(&mut canvas, ctx, self.is_host);
        }

        for (index, piece_move) in self.past_moves.iter().enumerate() {
            let text_position = glam::Vec2::new(900.0, 105.0 + ((index as f32) * 40.0));
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
        if self.start.is_some() {
            let color_to_string = match self.start.as_ref().unwrap().is_white {
                true => "White",
                false => "Black",
            };
            let text = Text::new(String::from("You are playing as: ") + color_to_string);
            text.draw(
                &mut canvas,
                graphics::DrawParam::default().dest(glam::Vec2::new(500.0, 70.0)),
            );
        }
        canvas.set_default_sampler();

        // Draw a stroked rectangle mesh.
        canvas.draw(&self.rect, graphics::DrawParam::default());

        // Draw some pre-made meshes

        // Finished drawing, show it all on the screen!
        canvas.finish(ctx)?;

        // function to handle the incoming or outgoing network packets
        if let Some(stream) = &mut self.client_stream {
            let mut buf = [0u8; 1024];
            match stream.read(&mut buf) {
                Ok(size) => {
                    if size > 0 {
                        println!("Received data: {}", size);
                        println!("Is host machine: {}", self.is_host.unwrap());
                        match NetworkMove::try_from(&buf[..]) {
                            Ok(piece_move) => {
                                println!("Received move: {:?}", piece_move);
                                let from = Square {
                                    file: File::from_idx(piece_move.from.0),
                                    rank: Rank::from_idx(piece_move.from.1),
                                };
                                let to = Square {
                                    file: File::from_idx(piece_move.to.0),
                                    rank: Rank::from_idx(piece_move.to.1),
                                };
                                let chess_move = Move::new(
                                    from,
                                    to,
                                    promotion_piece_to_piece(piece_move.promotion),
                                );
                                if self.is_host.is_some_and(|f| f == false) {
                                    // client will always ack the move
                                    let return_move_package = Ack {
                                        ok: true,
                                        end_state: None,
                                    };
                                    let return_move_package_bytes: Vec<u8> =
                                        return_move_package.try_into().unwrap();
                                    self.client_stream
                                        .as_ref()
                                        .unwrap()
                                        .write(&return_move_package_bytes)
                                        .unwrap();
                                    match self.board.make_move(chess_move) {
                                        Ok(_) => {
                                            self.past_moves
                                                .insert(0, (self.board.side(), chess_move));
                                            let pieces: [Option<(Piece, PieceColor)>; 64] =
                                                self.board.get_all_pieces();
                                            let piece_images: [Option<graphics::Image>; 64] =
                                                pieces.map(|piece| match piece {
                                                    Some(p) => Some(
                                                        graphics::Image::from_path(
                                                            ctx,
                                                            piece_to_image(p),
                                                        )
                                                        .unwrap(),
                                                    ),
                                                    None => None,
                                                });
                                            self.piece_images = piece_images;
                                        }
                                        Err(e) => println!("Error making move: {}", e),
                                    }
                                }
                                if self.is_host.is_some_and(|f| f == true) {
                                    let legal_moves = self.board.get_moves(chess_move.from());
                                    let is_legal =
                                        legal_moves.as_ref().unwrap().contains(&chess_move);
                                    let end_state = if self.board.get_game_result()
                                        == ChessResult::InProgress
                                    {
                                        None
                                    } else if self.board.get_game_result()
                                        == (ChessResult::Checkmate {
                                            winner: PieceColor::White,
                                        })
                                        || self.board.get_game_result()
                                            == (ChessResult::Checkmate {
                                                winner: PieceColor::Black,
                                            })
                                    {
                                        Some(GameState::CheckMate)
                                    } else if self.board.get_game_result() == ChessResult::Draw {
                                        Some(GameState::Draw)
                                    } else {
                                        None
                                    };
                                    let return_move_package = Ack {
                                        ok: is_legal,
                                        end_state,
                                    };
                                    let return_move_package_bytes: Vec<u8> =
                                        return_move_package.try_into().unwrap();
                                    self.client_stream
                                        .as_ref()
                                        .unwrap()
                                        .write(&return_move_package_bytes)
                                        .unwrap();

                                    match self.board.make_move(chess_move) {
                                        Ok(_) => {
                                            let pieces: [Option<(Piece, PieceColor)>; 64] =
                                                self.board.get_all_pieces();
                                            let piece_images: [Option<graphics::Image>; 64] =
                                                pieces.map(|piece| match piece {
                                                    Some(p) => Some(
                                                        graphics::Image::from_path(
                                                            ctx,
                                                            piece_to_image(p),
                                                        )
                                                        .unwrap(),
                                                    ),
                                                    None => None,
                                                });
                                            self.piece_images = piece_images;

                                            self.past_moves
                                                .insert(0, (self.board.side(), chess_move));
                                        }
                                        Err(e) => println!("Error making move: {}", e),
                                    }
                                }
                            }
                            Err(e) => println!("Error parsing move: {}", e),
                        }
                        match Ack::try_from(&buf[..]) {
                            Ok(ack) => {
                                println!("Received ack: {:?}", ack);
                                if self.is_host.is_some_and(|f| f == false) {
                                    if ack.ok {
                                        match self.board.make_move(self.pending_chess_move.unwrap())
                                        {
                                            Ok(_) => {
                                                self.past_moves.insert(
                                                    0,
                                                    (
                                                        self.board.side(),
                                                        self.pending_chess_move.unwrap(),
                                                    ),
                                                );
                                                self.pending_chess_move = None;
                                                let pieces: [Option<(Piece, PieceColor)>; 64] =
                                                    self.board.get_all_pieces();
                                                let piece_images: [Option<graphics::Image>; 64] =
                                                    pieces.map(|piece| match piece {
                                                        Some(p) => Some(
                                                            graphics::Image::from_path(
                                                                ctx,
                                                                piece_to_image(p),
                                                            )
                                                            .unwrap(),
                                                        ),
                                                        None => None,
                                                    });
                                                self.piece_images = piece_images;
                                            }
                                            Err(e) => println!("Error making move: {}", e),
                                        }
                                    }
                                } else if self.is_host.is_some_and(|f| f == true) {
                                    match self.board.make_move(self.pending_chess_move.unwrap()) {
                                        Ok(_) => {
                                            self.past_moves.insert(
                                                0,
                                                (
                                                    self.board.side(),
                                                    self.pending_chess_move.unwrap(),
                                                ),
                                            );
                                            self.pending_chess_move = None;
                                            let pieces: [Option<(Piece, PieceColor)>; 64] =
                                                self.board.get_all_pieces();
                                            let piece_images: [Option<graphics::Image>; 64] =
                                                pieces.map(|piece| match piece {
                                                    Some(p) => Some(
                                                        graphics::Image::from_path(
                                                            ctx,
                                                            piece_to_image(p),
                                                        )
                                                        .unwrap(),
                                                    ),
                                                    None => None,
                                                });
                                            self.piece_images = piece_images;
                                        }
                                        Err(e) => println!("Error making move: {}", e),
                                    }
                                }
                            }
                            Err(e) => println!("Error parsing ack: {}", e),
                        }
                        match Start::try_from(&buf[..]) {
                            Ok(start) => {
                                println!("Received start: {:?}", start);
                                if self.is_host.is_some_and(|f| f == true) {
                                    // selected color will always remain the same for the host
                                    // however if client has chosen same as host, then client color will be opposite
                                    let client_is_white = if start.is_white
                                        && self
                                            .selected_color
                                            .is_some_and(|f| f == PieceColor::White)
                                    {
                                        false
                                    } else if !start.is_white
                                        && self
                                            .selected_color
                                            .is_some_and(|f| f == PieceColor::Black)
                                    {
                                        true
                                    } else {
                                        start.is_white
                                    };
                                    let return_start_package = Start {
                                        is_white: client_is_white,
                                        name: start.name,
                                        fen: start.fen,
                                        time: start.time,
                                        inc: start.inc,
                                    };
                                    let return_start_package_bytes: Vec<u8> =
                                        return_start_package.try_into().unwrap();

                                    self.start = Some(Start {
                                        is_white: self.selected_color.unwrap() == PieceColor::White,
                                        name: None,
                                        fen: None,
                                        time: None,
                                        inc: None,
                                    });
                                    self.client_stream
                                        .as_ref()
                                        .unwrap()
                                        .write(&return_start_package_bytes)
                                        .unwrap();
                                } else {
                                    // client receives start package from host (after sending it once)
                                    self.selected_color = Some(if start.is_white {
                                        PieceColor::Black
                                    } else {
                                        PieceColor::White
                                    });
                                    self.start = Some(start);
                                }
                            }
                            Err(e) => println!("Error parsing start: {}", e),
                        }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No data available right now, just continue
                }
                Err(e) => println!("Error reading from stream: {}", e),
            }
        }

        // Draw an image with some options, and different filter modes.

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
