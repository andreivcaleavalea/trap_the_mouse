use common::{convert_to_i32, send_ok, AppError, Position};
use std::{
    io::{ErrorKind, Read, Write},
    net::TcpStream,
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};

pub enum Screen {
    MainMenu,
    WaitingForPlayers,
    ConnectingError,
    GameBoard,
}

pub enum State {
    WaitTryConnect,
    Connect,
    WaitToStart,
    GetTurn,
    GetMouse,
    GetWalls,
    Play,
    Finished,
    PlayAgain,
}

pub struct Game {
    pub screen: Screen,
    pub show_input_1: bool,
    pub show_input_2: bool,
    pub room_input: String,
    pub stream: Option<TcpStream>,
    pub player_text: Option<String>,
    pub player: i32,
    pub mouse_texture: Option<eframe::egui::TextureHandle>,
    pub initial_mouse: bool,
    pub mouse: Position,
    pub board: [[u8; 11]; 11],
    pub has_to_read: bool,
    pub win: bool,
    pub state: State,
    pub win_state: String,
    pub try_connect: bool,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            screen: Screen::MainMenu,
            show_input_1: false,
            show_input_2: false,
            room_input: String::from("-1"),
            stream: None,
            try_connect: false,
            player: -1,
            player_text: None,
            mouse_texture: None,
            initial_mouse: false,
            mouse: Position::new(0, 0),
            board: [[0; 11]; 11],
            has_to_read: false,
            win: false,
            state: State::WaitTryConnect,
            win_state: String::new(),
        }
    }
}

impl Game {
    pub fn start_tcp(game: Arc<RwLock<Self>>) {
        {
            let mut game = game.write().unwrap();

            match TcpStream::connect("127.0.0.1:8080") {
                Ok(stream) => {
                    game.stream = Some(stream);
                }
                Err(e) => {
                    AppError::ConnectionError(e.to_string()).log();
                    game.screen = Screen::ConnectingError;
                    return;
                }
            }
        }
        thread::spawn(move || {
            loop {
                {
                    let mut game = game.write().unwrap();

                    match game.state {
                        State::WaitTryConnect => {
                            if game.try_connect {
                                game.state = State::Connect;
                            }
                        }
                        State::Connect => {
                            let room_id = convert_to_i32(&game.room_input);

                            match game
                                .stream
                                .as_ref()
                                .unwrap()
                                .write_all(room_id.to_string().as_bytes())
                            {
                                Ok(_) => {}
                                Err(e) => {
                                    AppError::WriteError(e.to_string()).log();
                                }
                            }

                            let mut buffer = [0; 1024];

                            match game.stream.as_ref().unwrap().read(&mut buffer) {
                                Ok(0) => {
                                    game.screen = Screen::ConnectingError;

                                    return;
                                }
                                Ok(n) => {
                                    let response = String::from_utf8_lossy(&buffer[..n]);
                                    game.room_input = response.to_string();
                                    send_ok(&game.stream);

                                    game.screen = Screen::WaitingForPlayers;

                                    game.state = State::WaitToStart;

                                    println!("Te-ai conectat in camera: {}", game.room_input);
                                }
                                Err(e) => {
                                    AppError::ReadError(e.to_string()).log();
                                }
                            }
                        }
                        State::WaitToStart => {
                            let mut buffer = [0; 1024];
                            match game.stream.as_ref().unwrap().read(&mut buffer) {
                                Ok(0) => {
                                    game.screen = Screen::ConnectingError;
                                    break;
                                }
                                Ok(n) => {
                                    let message = String::from_utf8_lossy(&buffer[..n]);
                                    println!("START! {}", message);
                                    game.stream.as_ref().unwrap().flush().expect("Err flush");

                                    game.screen = Screen::GameBoard;
                                    send_ok(&game.stream);
                                    game.state = State::GetTurn;
                                }
                                Err(e) if e.kind() == ErrorKind::WouldBlock => {}
                                Err(e) => {
                                    AppError::ReadError(e.to_string()).log();
                                }
                            }
                        }
                        State::GetTurn => {
                            let mut buffer = [0; 1024];
                            match game.stream.as_ref().unwrap().read(&mut buffer) {
                                Ok(0) => {
                                    game.screen = Screen::ConnectingError;
                                    break;
                                }
                                Ok(n) => {
                                    game.player_text =
                                        Some(String::from_utf8_lossy(&buffer[..n]).to_string());
                                    if game.player_text.as_ref().unwrap().contains("soarecele") {
                                        game.player = 1;
                                    } else {
                                        game.player = 2;
                                        game.has_to_read = true;
                                    }
                                    send_ok(&game.stream);
                                    game.state = State::GetMouse;
                                    println!("Player {}", game.player_text.as_ref().unwrap());
                                }
                                Err(e) if e.kind() == ErrorKind::WouldBlock => {}
                                Err(e) => {
                                    AppError::ReadError(e.to_string()).log();
                                }
                            }
                        }
                        State::GetMouse => {
                            let mut buffer = [0; 1024];

                            let mut row = 0;
                            let mut col = 0;
                            match game.stream.as_ref().unwrap().read(&mut buffer) {
                                Ok(0) => {
                                    game.screen = Screen::ConnectingError;
                                    break;
                                }
                                Ok(n) => {
                                    let msg = String::from_utf8_lossy(&buffer[..n]);
                                    let parts: Vec<&str> = msg.split(',').collect();
                                    if parts.len() >= 2 {
                                        if let Ok(r) = parts[0].parse::<usize>() {
                                            row = r;
                                        }
                                        if let Ok(c) = parts[1].parse::<usize>() {
                                            col = c;
                                        }
                                        game.mouse = Position::new(row, col);
                                        game.board[row][col] = 1;
                                        println!("Mouse initial la: {}, {}", row, col);
                                        game.initial_mouse = true;
                                        send_ok(&game.stream);

                                        game.state = State::GetWalls;
                                    } else {
                                        println!("Mesaj invalid: {}", msg);
                                    }
                                }
                                Err(e) => {
                                    AppError::ReadError(e.to_string()).log();
                                }
                            }
                        }
                        State::GetWalls => {
                            let mut buffer = [0; 20];
                            let result = game.stream.as_ref().unwrap().read(&mut buffer);
                            match result {
                                Ok(n) if n > 0 => {
                                    game.board[buffer[0] as usize][buffer[1] as usize] = 2;
                                    game.board[buffer[2] as usize][buffer[3] as usize] = 2;
                                    game.board[buffer[4] as usize][buffer[5] as usize] = 2;
                                    game.board[buffer[6] as usize][buffer[7] as usize] = 2;
                                    game.board[buffer[8] as usize][buffer[9] as usize] = 2;

                                    game.board[buffer[10] as usize][buffer[11] as usize] = 2;
                                    game.board[buffer[12] as usize][buffer[13] as usize] = 2;
                                    game.board[buffer[14] as usize][buffer[15] as usize] = 2;
                                    game.board[buffer[16] as usize][buffer[17] as usize] = 2;
                                    game.board[buffer[18] as usize][buffer[19] as usize] = 2;
                                    game.state = State::Play;
                                }
                                Ok(0) => {
                                    game.screen = Screen::ConnectingError;
                                    return;
                                }
                                Ok(_) => {}
                                Err(e) => {
                                    AppError::ReadError(e.to_string()).log();
                                }
                            }
                            send_ok(&game.stream);
                        }
                        State::Play => {
                            let mut buffer: [u8; 2] = [0; 2];
                            match game.stream.as_ref().unwrap().set_nonblocking(true) {
                                Ok(_) => {}
                                Err(e) => {
                                    AppError::StreamUnavailable(e.to_string()).log();
                                }
                            }
                            match game.stream.as_ref().unwrap().read(&mut buffer) {
                                Ok(n) if n > 0 => {
                                    let nr1 = buffer[0];
                                    let nr2 = buffer[1];
                                    println!("Mutare primită: {}, {}", nr1, nr2);
                                    if buffer[0] == b'w' && buffer[1] == b'i' {
                                        let mut buffer = [0; 2];
                                        buffer[0] = b'w';
                                        buffer[1] = b'i';
                                        match game.stream.as_ref().unwrap().write(&buffer) {
                                            Ok(_) => {}
                                            Err(e) => {
                                                AppError::WriteError(e.to_string()).log();
                                            }
                                        }
                                        game.win_state = String::from("AI CASTIGAT!");
                                        game.win = true;
                                        game.state = State::Finished;
                                    } else if buffer[0] == b'l' && buffer[1] == b'o' {
                                        let mut buffer = [0; 2];
                                        buffer[0] = b'l';
                                        buffer[1] = b'o';
                                        match game.stream.as_ref().unwrap().write(&buffer) {
                                            Ok(_) => {}
                                            Err(e) => {
                                                AppError::WriteError(e.to_string()).log();
                                            }
                                        }
                                        game.win_state = String::from("AI PIERDUT!");
                                        game.win = true;
                                        game.state = State::Finished;
                                    } else if buffer[0] == b'y' && buffer[1] == b'e' {
                                        game.win = true;
                                        game.win_state = String::from(
                                            "          AI CASTIGAT \n PLAYERUL S-A DECONECTAT",
                                        );
                                        game.state = State::Finished;
                                    } else if game.player == 2 {
                                        let temp = Position::new_from_pos(&game.mouse);
                                        game.board[temp.x][temp.y] = 0;
                                        game.board[nr1 as usize][nr2 as usize] = 1;
                                        game.mouse = Position::new(nr1 as usize, nr2 as usize);
                                    } else {
                                        game.board[nr1 as usize][nr2 as usize] = 2;
                                    }

                                    game.has_to_read = false;
                                    if game.player == 1
                                        && !check_any_left_move(&game.board, &game.mouse)
                                    {
                                        let mut buffer = [0; 2];
                                        buffer[0] = b'l';
                                        buffer[1] = b'o';
                                        if let Err(e) = game.stream.as_ref().unwrap().write(&buffer)
                                        {
                                            AppError::WriteError(e.to_string()).log();
                                        }
                                        game.win = true;
                                        game.win_state = String::from("AI PIERDUT!");
                                        game.state = State::Finished;
                                        // break;
                                    }
                                }
                                Ok(0) => {
                                    game.win = true;
                                    game.win_state = String::from(
                                        "          AI CASTIGAT \n PLAYERUL S-A DECONECTAT",
                                    );
                                    game.state = State::Finished;
                                }
                                Ok(_) => {}
                                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                                    // thread::sleep(Duration::from_millis(100));
                                }
                                Err(e) => {
                                    AppError::ReadError(e.to_string()).log();
                                }
                            }
                        }
                        State::Finished => {}
                        State::PlayAgain => match TcpStream::connect("127.0.0.1:8080") {
                            Ok(stream) => {
                                game.stream = Some(stream);
                                game.state = State::WaitTryConnect;
                                game.screen = Screen::MainMenu;
                                println!("Conectare dinou reusita");
                            }
                            Err(e) => {
                                AppError::ConnectionError(e.to_string()).log();
                                break;
                            }
                        },
                    }

                    drop(game);
                }
                thread::sleep(Duration::from_millis(100));
            }
        });
    }
}

pub fn check_move(
    r: usize,
    c: usize,
    mouse: &Position,
    player: u8,
    board: &[[u8; 11]; 11],
) -> bool {
    if player == 2 {
        return board[r][c] == 0;
    }
    if board[r][c] != 0 {
        return false;
    }
    let mut valid_moves: Vec<(i32, i32)> = Vec::new();
    let mouse_x = mouse.x as i32;
    let mouse_y = mouse.y as i32;
    let r = r as i32;
    let c = c as i32;
    if mouse_x % 2 == 0 {
        valid_moves.push((mouse_x - 1, mouse_y - 1));
        valid_moves.push((mouse_x - 1, mouse_y));
        valid_moves.push((mouse_x, mouse_y - 1));
        valid_moves.push((mouse_x, mouse_y + 1));
        valid_moves.push((mouse_x + 1, mouse_y - 1));
        valid_moves.push((mouse_x + 1, mouse_y));
    } else {
        valid_moves.push((mouse_x - 1, mouse_y));
        valid_moves.push((mouse_x - 1, mouse_y + 1));
        valid_moves.push((mouse_x, mouse_y - 1));
        valid_moves.push((mouse_x, mouse_y + 1));
        valid_moves.push((mouse_x + 1, mouse_y));
        valid_moves.push((mouse_x + 1, mouse_y + 1));
    }
    if valid_moves.iter().any(|&(x, y)| x == r && y == c) {
        return true;
    }
    println!("Mutarea {}, {} este invalidă!", r, c);
    false
}

pub fn check_any_left_move(board: &[[u8; 11]; 11], mouse: &Position) -> bool {
    let mut valid_moves: Vec<(i32, i32)> = Vec::new();
    let mouse_x = mouse.x as i32;
    let mouse_y = mouse.y as i32;

    if mouse_x % 2 == 0 {
        valid_moves.push((mouse_x - 1, mouse_y - 1));
        valid_moves.push((mouse_x - 1, mouse_y));
        valid_moves.push((mouse_x, mouse_y - 1));
        valid_moves.push((mouse_x, mouse_y + 1));
        valid_moves.push((mouse_x + 1, mouse_y - 1));
        valid_moves.push((mouse_x + 1, mouse_y));
    } else {
        valid_moves.push((mouse_x - 1, mouse_y));
        valid_moves.push((mouse_x - 1, mouse_y + 1));
        valid_moves.push((mouse_x, mouse_y - 1));
        valid_moves.push((mouse_x, mouse_y + 1));
        valid_moves.push((mouse_x + 1, mouse_y));
        valid_moves.push((mouse_x + 1, mouse_y + 1));
    }

    let mut ok = false;
    for pos in valid_moves {
        if pos.0 >= 0 && pos.1 <= 10 && board[pos.0 as usize][pos.1 as usize] == 0 {
            ok = true;
            break;
        }
    }
    ok
}

pub fn send_move(game: &mut Game, row: usize, col: usize) {
    if game.player == 1 {
        game.board[game.mouse.x][game.mouse.y] = 0;
        game.board[row][col] = 1;
        game.mouse.x = row;
        game.mouse.y = col;
    } else {
        game.board[row][col] = 2;
    }
    let mut buffer: [u8; 2] = [0; 2];
    if (game.player == 1) && (row == 0 || row == 10 || col == 0 || col == 10) {
        buffer[0] = b'w';
        buffer[1] = b'i';
    } else {
        buffer[0] = row as u8;
        buffer[1] = col as u8;
    }
    match game.stream.as_ref().unwrap().write_all(&buffer) {
        Ok(_) => {}
        Err(e) => {
            AppError::WriteError(e.to_string()).log();
        }
    }
    println!("S-a trimis mutarea {},{}", buffer[0], buffer[1]);
}
