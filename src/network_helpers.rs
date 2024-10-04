use crate::{piece_to_image, promotion_piece_to_piece, State};
use chess_networking::{Ack, GameState, Move as NetworkMove, PromotionPiece, Start};
use dexterws_chess::game::{
    Color as PieceColor, File, GameResult as ChessResult, Move, Piece, Rank, Square,
};
use ggez::*;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};

pub fn listen_for_connections(state: &mut State) {
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

pub fn connect_to_host(address: String, state: &mut State) {
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

pub fn handle_incoming_packages(ctx: &mut Context, state: &mut State) {
    if let Some(stream) = &mut state.client_stream {
        let mut buf = [0u8; 1024];
        match stream.read(&mut buf) {
            Ok(size) => {
                if size > 0 {
                    match NetworkMove::try_from(&buf[..]) {
                        Ok(piece_move) => {
                            if piece_move.forfeit {
                                state.game_has_ended = true;
                                state.client_stream = None;
                                return;
                            }
                            if piece_move.offer_draw {
                                state.offer_draw_received = true;
                                return;
                            }

                            let from = Square {
                                file: File::from_idx(piece_move.from.0),
                                rank: Rank::from_idx(piece_move.from.1),
                            };
                            let to = Square {
                                file: File::from_idx(piece_move.to.0),
                                rank: Rank::from_idx(piece_move.to.1),
                            };
                            let chess_move =
                                Move::new(from, to, promotion_piece_to_piece(piece_move.promotion));
                            if state.is_host.is_some_and(|f| f == false) {
                                // client will always ack the move
                                let return_move_package = Ack {
                                    ok: true,
                                    end_state: None,
                                };
                                let return_move_package_bytes: Vec<u8> =
                                    return_move_package.try_into().unwrap();
                                state
                                    .client_stream
                                    .as_ref()
                                    .unwrap()
                                    .write(&return_move_package_bytes)
                                    .unwrap();
                                match state.board.make_move(chess_move) {
                                    Ok(_) => {
                                        state
                                            .past_moves
                                            .insert(0, (state.board.side(), chess_move));
                                        let pieces: [Option<(Piece, PieceColor)>; 64] =
                                            state.board.get_all_pieces();
                                        let piece_images: [Option<graphics::Image>; 64] = pieces
                                            .map(|piece| match piece {
                                                Some(p) => Some(
                                                    graphics::Image::from_path(
                                                        ctx,
                                                        piece_to_image(p),
                                                    )
                                                    .unwrap(),
                                                ),
                                                None => None,
                                            });
                                        state.piece_images = piece_images;
                                    }
                                    Err(e) => println!("Error making move: {}", e),
                                }
                            }
                            if state.is_host.is_some_and(|f| f == true) {
                                let legal_moves = state.board.get_moves(chess_move.from());
                                let is_legal = legal_moves.as_ref().unwrap().contains(&chess_move);

                                if is_legal {
                                    match state.board.make_move(chess_move) {
                                        Ok(_) => {
                                            let pieces: [Option<(Piece, PieceColor)>; 64] =
                                                state.board.get_all_pieces();
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
                                            state.piece_images = piece_images;

                                            state
                                                .past_moves
                                                .insert(0, (state.board.side(), chess_move));
                                        }
                                        Err(e) => println!("Error making move: {}", e),
                                    }
                                }
                                let end_state =
                                    if state.board.get_game_result() == ChessResult::InProgress {
                                        None
                                    } else if state.board.get_game_result()
                                        == (ChessResult::Checkmate {
                                            winner: PieceColor::White,
                                        })
                                        || state.board.get_game_result()
                                            == (ChessResult::Checkmate {
                                                winner: PieceColor::Black,
                                            })
                                    {
                                        Some(GameState::CheckMate)
                                    } else if state.board.get_game_result() == ChessResult::Draw {
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
                                state
                                    .client_stream
                                    .as_ref()
                                    .unwrap()
                                    .write(&return_move_package_bytes)
                                    .unwrap();
                            }
                        }
                        Err(e) => println!("Error parsing move: {}", e),
                    }
                    match Ack::try_from(&buf[..]) {
                        Ok(ack) => {
                            if state.offer_draw_sent {
                                if ack.ok {
                                    state.game_has_ended = true;
                                    state.client_stream = None;
                                    state.offer_draw_sent = false;
                                } else {
                                    state.offer_draw_sent = false;
                                }
                                return;
                            }
                            if state.is_host.is_some_and(|f| f == false) {
                                if ack.end_state.is_some() {
                                    state.game_has_ended = true;
                                    // Close the connection

                                    state.client_stream = None;
                                }
                                if ack.ok {
                                    match state.board.make_move(state.pending_chess_move.unwrap()) {
                                        Ok(_) => {
                                            state.past_moves.insert(
                                                0,
                                                (
                                                    state.board.side(),
                                                    state.pending_chess_move.unwrap(),
                                                ),
                                            );
                                            state.pending_chess_move = None;
                                            let pieces: [Option<(Piece, PieceColor)>; 64] =
                                                state.board.get_all_pieces();
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
                                            state.piece_images = piece_images;
                                        }
                                        Err(e) => println!("Error making move: {}", e),
                                    }
                                }
                            } else if state.is_host.is_some_and(|f| f == true) {
                                match state.board.make_move(state.pending_chess_move.unwrap()) {
                                    Ok(_) => {
                                        state.past_moves.insert(
                                            0,
                                            (state.board.side(), state.pending_chess_move.unwrap()),
                                        );
                                        state.pending_chess_move = None;
                                        let pieces: [Option<(Piece, PieceColor)>; 64] =
                                            state.board.get_all_pieces();
                                        let piece_images: [Option<graphics::Image>; 64] = pieces
                                            .map(|piece| match piece {
                                                Some(p) => Some(
                                                    graphics::Image::from_path(
                                                        ctx,
                                                        piece_to_image(p),
                                                    )
                                                    .unwrap(),
                                                ),
                                                None => None,
                                            });
                                        state.piece_images = piece_images;
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
                            if state.is_host.is_some_and(|f| f == true) {
                                // selected color will always remain the same for the host
                                // however if client has chosen same as host, then client color will be opposite
                                let client_is_white = if start.is_white
                                    && state.selected_color.is_some_and(|f| f == PieceColor::White)
                                {
                                    false
                                } else if !start.is_white
                                    && state.selected_color.is_some_and(|f| f == PieceColor::Black)
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

                                state.start = Some(Start {
                                    is_white: state.selected_color.unwrap() == PieceColor::White,
                                    name: None,
                                    fen: None,
                                    time: None,
                                    inc: None,
                                });
                                state
                                    .client_stream
                                    .as_ref()
                                    .unwrap()
                                    .write(&return_start_package_bytes)
                                    .unwrap();
                            } else {
                                // client receives start package from host (after sending it once)
                                state.selected_color = Some(if start.is_white {
                                    PieceColor::Black
                                } else {
                                    PieceColor::White
                                });
                                state.start = Some(start);
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
}
