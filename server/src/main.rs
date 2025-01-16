use std::cmp;
use std::{
    collections::{HashSet, VecDeque},
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, RwLock},
    thread,
};

use common::{AppError, Position};
use rand::Rng;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("Serverul rulează pe 127.0.0.1:8080");

    let rooms_manager = Arc::new(RwLock::new(RoomsManager::new()));

    for stream in listener.incoming() {
        let rooms_manager = Arc::clone(&rooms_manager);
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream, rooms_manager.clone());
                });
            }
            Err(e) => {
                eprintln!("Eroare la acceptarea conexiunii: {}", e);
            }
        }
    }

    Ok(())
}

struct Room {
    player1: Option<TcpStream>,
    player2: Option<TcpStream>,
    is_full: bool,
    code: i8,
    is_taken: bool,
}

impl Room {
    fn new(code: i8) -> Self {
        Room {
            player1: None,
            player2: None,
            is_full: false,
            code,
            is_taken: false,
        }
    }

    fn add_to_room(&mut self, stream: &TcpStream) -> bool {
        if self.player1.is_none() {
            self.player1 = Some(stream.try_clone().unwrap());
            if self.player2.is_some() {
                self.is_full = true;
            }
            return true;
        } else if self.player2.is_none() {
            self.player2 = Some(stream.try_clone().unwrap());
            if self.player1.is_some() {
                self.is_full = true;
            }
            return true;
        }
        false
    }
}

struct RoomsManager {
    rooms: Vec<Room>,
}

impl RoomsManager {
    fn new() -> Self {
        RoomsManager { rooms: Vec::new() }
    }

    fn add_to_random_room(&mut self, stream: &TcpStream) -> i8 {
        for room in self.rooms.iter_mut() {
            if !room.is_full {
                room.add_to_room(stream);
                return room.code;
            }
        }

        self.rooms.push(Room::new(self.rooms.len() as i8));
        let room_id = self.rooms.len() - 1;
        self.rooms[room_id].add_to_room(stream);
        room_id as i8
    }

    fn add_to_specific_room(&mut self, stream: &TcpStream, code: i8) -> i8 {
        for room in self.rooms.iter_mut() {
            if room.code == code && !room.is_full {
                room.add_to_room(stream);
                return room.code;
            }
        }
        self.rooms.push(Room::new(code));
        let room_id = self.rooms.len() - 1;
        self.rooms[room_id].add_to_room(stream);
        code
    }

    fn remove_from_room(&mut self, code: usize) {
        for (index, room) in self.rooms.iter().enumerate() {
            if room.code == code as i8 {
                self.rooms.remove(index);
                break;
            }
        }
    }

    fn show_rooms(&self) {
        println!("Camere curente: ");
        for room in self.rooms.iter() {
            println!("Camera {}", room.code);
            println!("Player1: {:?}", room.player1);
            println!("Player2: {:?}", room.player2);
            println!("---------------------------");
        }
    }

    fn check_room(&mut self, code: i8) -> bool {
        for room in self.rooms.iter() {
            if room.code == code && room.is_full {
                return true;
            }
        }
        false
    }
}

fn handle_client(mut stream: TcpStream, rooms_manager: Arc<RwLock<RoomsManager>>) {
    let mut room_id: i8 = -1;

    let mut buffer: [u8; 1024] = [0; 1024];
    match stream.read(&mut buffer) {
        Ok(0) => {
            println!("Clientul s-a deconectat inainte sa fie pus intr-o camera!!!");
            return;
        }
        Ok(n) => {
            let code = String::from_utf8_lossy(&buffer[..n]);
            let code = code.trim();

            match code.parse::<i8>() {
                Ok(num) => {
                    println!("Cod camera de la client: {}", num);
                    room_id = num;
                }
                Err(e) => println!("Eroare la parsare: {}", e),
            }
        }
        Err(e) => {
            AppError::ReadError(e.to_string()).log();
            return;
        }
    }

    match room_id {
        -1 => {
            let mut rooms_manager = rooms_manager.write().unwrap();
            room_id = rooms_manager.add_to_random_room(&stream);

            println!(
                "Clientul {:?} a fost adaugat in camera: {}",
                stream, room_id
            );

            match stream.write_all(room_id.to_string().as_bytes()) {
                Ok(_) => {}
                Err(e) => {
                    AppError::WriteError(e.to_string()).log();
                }
            }
            stream.flush().unwrap();
            rooms_manager.show_rooms();
        }
        -2 => {
            match stream.write_all(room_id.to_string().as_bytes()) {
                Ok(_) => {}
                Err(e) => {
                    AppError::WriteError(e.to_string()).log();
                }
            }
            read_ok(&Some(stream.try_clone().unwrap()));
            stream.flush().unwrap();
            let stream_for_computer = stream.try_clone().expect("Nu s-a putut clona stream-ul");

            handle_room_computer(room_id, Some(stream_for_computer));
            return;
        }
        _ => {
            let mut rooms_manager = rooms_manager.write().unwrap();
            room_id = rooms_manager.add_to_specific_room(&stream, room_id);

            println!(
                "Clientul {:?} a fost adaugat in camera: {}",
                stream, room_id
            );

            match stream.write_all(room_id.to_string().as_bytes()) {
                Ok(_) => {}
                Err(e) => {
                    AppError::WriteError(e.to_string()).log();
                }
            }
            stream.flush().unwrap();
            rooms_manager.show_rooms();
        }
    }
    loop {
        {
            let mut rooms_manager = rooms_manager.write().unwrap();
            if rooms_manager.check_room(room_id) {
                println!("Camera este plina {}", room_id);
                break;
            }
        }
    }
    read_ok(&Some(stream.try_clone().unwrap()));

    let message = format!("Jocul a inceput in camera! {}", room_id);
    match stream.write_all(message.as_bytes()) {
        Ok(_) => {}
        Err(e) => {
            AppError::WriteError(e.to_string()).log();
        }
    }
    stream.flush().unwrap();
    read_ok(&Some(stream));
    handle_room(room_id, rooms_manager);
}

fn handle_room(room_id: i8, rooms_manager: Arc<RwLock<RoomsManager>>) {
    let mut player1: Option<TcpStream> = None;
    let mut player2: Option<TcpStream> = None;

    {
        let mut rooms_manager = rooms_manager.write().unwrap();
        for room in rooms_manager.rooms.iter_mut() {
            if room.code == room_id {
                if room.is_taken {
                    return;
                }
                room.is_taken = true;
                player1 = room.player1.as_ref().and_then(|s| s.try_clone().ok());
                player2 = room.player2.as_ref().and_then(|s| s.try_clone().ok());
                break;
            }
        }
    }
    player1.as_ref().unwrap().flush().expect("Err flush");
    if let Err(e) = player1
        .as_ref()
        .unwrap()
        .write_all(String::from("Tu esti soarecele !").as_bytes())
    {
        AppError::WriteError(e.to_string()).log();
    }
    player1.as_ref().unwrap().flush().unwrap();
    // println!("asteptam ok de la player1");
    read_ok(&player1);

    player2.as_ref().unwrap().flush().expect("Err flush");
    if let Err(e) = player2
        .as_ref()
        .unwrap()
        .write_all(String::from("Tu pui zidurile!").as_bytes())
    {
        AppError::WriteError(e.to_string()).log();
    }
    player2.as_ref().unwrap().flush().unwrap();
    // println!("asteptam ok de la player1");
    read_ok(&player2);

    let (row, col) = send_mouse_pos(&player1, &player2);
    read_ok(&player1);
    read_ok(&player2);

    let walls = generate_wall_positions((row as u8, col as u8));

    let mut buffer: [u8; 20] = [0; 20];
    buffer[0] = walls[0].0;
    buffer[1] = walls[0].1;
    buffer[2] = walls[1].0;
    buffer[3] = walls[1].1;
    buffer[4] = walls[2].0;
    buffer[5] = walls[2].1;
    buffer[6] = walls[3].0;
    buffer[7] = walls[3].1;
    buffer[8] = walls[4].0;
    buffer[9] = walls[4].1;
    buffer[10] = walls[5].0;
    buffer[11] = walls[5].1;
    buffer[12] = walls[6].0;
    buffer[13] = walls[6].1;
    buffer[14] = walls[7].0;
    buffer[15] = walls[7].1;
    buffer[16] = walls[8].0;
    buffer[17] = walls[8].1;
    buffer[18] = walls[9].0;
    buffer[19] = walls[9].1;

    if let Err(e) = player1.as_ref().unwrap().write_all(&buffer) {
        AppError::WriteError(e.to_string()).log();
    }
    read_ok(&player1);

    if let Err(e) = player2.as_ref().unwrap().write_all(&buffer) {
        AppError::WriteError(e.to_string()).log();
    }
    read_ok(&player2);

    println!("Incepem jocul!");

    let mut close = false;

    loop {
        let mut buffer: [u8; 2] = [0; 2];
        match player1.as_ref().unwrap().read(&mut buffer) {
            Ok(n) if n > 0 => {
                if buffer[0] == 119 && buffer[1] == b'i' {
                    buffer[0] = b'l';
                    buffer[1] = b'o';
                    close = true;
                } else if buffer[0] == b'l' && buffer[1] == b'o' {
                    buffer[0] = b'w';
                    buffer[1] = b'i';
                    close = true;
                }
                // println!("--> P2.send: {}, {}", buffer[0], buffer[1]);
                let _n = player2
                    .as_ref()
                    .unwrap()
                    .write(&buffer)
                    .expect("Eroare la scriere");
            }
            Ok(0) => {
                println!("Clientul s-a deconectat!");

                let mut buffer = [0; 2];
                buffer[0] = b'y';
                buffer[1] = b'e';

                let _n = player2
                    .as_ref()
                    .unwrap()
                    .write(&buffer)
                    .expect("Eroare la scriere");

                let _ = player1.as_ref().unwrap().shutdown(std::net::Shutdown::Both);
                let _ = player2.as_ref().unwrap().shutdown(std::net::Shutdown::Both);
                {
                    let mut rooms_manager = rooms_manager.write().unwrap();
                    rooms_manager.remove_from_room(room_id as usize);
                }
                return;
            }
            Ok(_) => {}
            Err(e) => {
                AppError::ReadError(e.to_string()).log();
            }
        }
        let mut buffer: [u8; 2] = [0; 2];

        match player2.as_ref().unwrap().read(&mut buffer) {
            Ok(n) if n > 0 => {
                if buffer[0] == b'w' && buffer[1] == b'i' {
                    buffer[0] = b'l';
                    buffer[1] = b'o';
                    close = true;
                } else if buffer[0] == b'l' && buffer[1] == b'o' {
                    buffer[0] = b'w';
                    buffer[1] = b'i';
                    close = true;
                }
                // println!("<-- P1.send: {}, {}", buffer[0], buffer[1]);
                let _n = player1
                    .as_ref()
                    .unwrap()
                    .write(&buffer)
                    .expect("Eroare la scriere");
            }
            Ok(0) => {
                println!("Clientul s-a deconectat!");

                let mut buffer = [0; 2];
                buffer[0] = b'y';
                buffer[1] = b'e';

                let _n = player1
                    .as_ref()
                    .unwrap()
                    .write(&buffer)
                    .expect("Eroare la scriere");
                if let Err(e) = player1.as_ref().unwrap().shutdown(std::net::Shutdown::Both) {
                    AppError::StreamUnavailable(e.to_string()).log();
                }
                if let Err(e) = player2.as_ref().unwrap().shutdown(std::net::Shutdown::Both) {
                    AppError::StreamUnavailable(e.to_string()).log();
                }
                {
                    let mut rooms_manager = rooms_manager.write().unwrap();
                    rooms_manager.remove_from_room(room_id as usize);
                }
                return;
            }
            Ok(_) => {}
            Err(e) => {
                AppError::ReadError(e.to_string()).log();
            }
        }
        if close {
            {
                let mut rooms_manager = rooms_manager.write().unwrap();
                rooms_manager.remove_from_room(room_id as usize);
            }
            break;
        }
    }
}

fn handle_room_computer(room_id: i8, stream: Option<TcpStream>) {
    let message = format!("Jocul a inceput in camera! {}", room_id);
    if let Err(e) = stream.as_ref().unwrap().write_all(message.as_bytes()) {
        AppError::WriteError(e.to_string()).log();
    }
    stream.as_ref().unwrap().flush().unwrap();
    read_ok(&stream);

    let role_msg = "Tu pui zidurile!";
    if let Err(e) = stream.as_ref().unwrap().write_all(role_msg.as_bytes()) {
        AppError::WriteError(e.to_string()).log();
    }
    if let Err(e) = stream.as_ref().unwrap().flush() {
        AppError::WriteError(e.to_string()).log();
    }

    read_ok(&stream);

    let (mut mouse_x, mut mouse_y) = send_mouse_pos_computer(&stream);
    stream
        .as_ref()
        .unwrap()
        .flush()
        .expect("Eroare la flush după trimiterea poziției mouse-ului");
    read_ok(&stream);

    let walls = generate_wall_positions((mouse_x as u8, mouse_y as u8));

    let mut buffer: [u8; 20] = [0; 20];
    buffer[0] = walls[0].0;
    buffer[1] = walls[0].1;
    buffer[2] = walls[1].0;
    buffer[3] = walls[1].1;
    buffer[4] = walls[2].0;
    buffer[5] = walls[2].1;
    buffer[6] = walls[3].0;
    buffer[7] = walls[3].1;
    buffer[8] = walls[4].0;
    buffer[9] = walls[4].1;
    buffer[10] = walls[5].0;
    buffer[11] = walls[5].1;
    buffer[12] = walls[6].0;
    buffer[13] = walls[6].1;
    buffer[14] = walls[7].0;
    buffer[15] = walls[7].1;
    buffer[16] = walls[8].0;
    buffer[17] = walls[8].1;
    buffer[18] = walls[9].0;
    buffer[19] = walls[9].1;

    if let Err(e) = stream.as_ref().unwrap().write_all(&buffer) {
        AppError::WriteError(e.to_string()).log();
    }

    read_ok(&stream);

    let mut board: [[u8; 11]; 11] = [[0; 11]; 11];
    board[mouse_x][mouse_y] = 1;

    board[walls[0].0 as usize][walls[0].1 as usize] = 2;
    board[walls[1].0 as usize][walls[1].1 as usize] = 2;
    board[walls[2].0 as usize][walls[2].1 as usize] = 2;
    board[walls[3].0 as usize][walls[3].1 as usize] = 2;
    board[walls[4].0 as usize][walls[4].1 as usize] = 2;
    board[walls[5].0 as usize][walls[5].1 as usize] = 2;
    board[walls[6].0 as usize][walls[6].1 as usize] = 2;
    board[walls[7].0 as usize][walls[7].1 as usize] = 2;
    board[walls[8].0 as usize][walls[8].1 as usize] = 2;
    board[walls[9].0 as usize][walls[9].1 as usize] = 2;

    loop {
        if !check_any_left_move(&board, &Position::new(mouse_x, mouse_y)) {
            buffer[0] = b'w';
            buffer[1] = b'i';

            if let Err(e) = stream.as_ref().unwrap().write_all(&buffer) {
                AppError::WriteError(e.to_string()).log();
            }

            break;
        }
        if let Some((row, col)) = find_shortest_path_to_border(&board, mouse_x, mouse_y) {
            board[mouse_x][mouse_y] = 0;
            board[row][col] = 1;
            mouse_x = row;
            mouse_y = col;
            if mouse_x == 0 || mouse_x == 10 || mouse_y == 0 || mouse_y == 10 {
                buffer[0] = b'l';
                buffer[1] = b'o';

                if let Err(e) = stream.as_ref().unwrap().write_all(&buffer) {
                    AppError::WriteError(e.to_string()).log();
                }
                break;
            }
            let move_bytes = [row as u8, col as u8];
            if let Err(e) = stream.as_ref().unwrap().write_all(&move_bytes) {
                AppError::WriteError(e.to_string()).log();
            }
            if let Err(e) = stream.as_ref().unwrap().flush() {
                AppError::WriteError(e.to_string()).log();
            }

            // println!("Mutarea computerului: {},{}", row, col);
        } else {
            AppError::UnexpectedResponse(String::from("Fara mutari posibile")).log();
            break;
        }

        let mut client_move: [u8; 2] = [0; 2];
        match stream.as_ref().unwrap().read_exact(&mut client_move) {
            Ok(()) => {
                if client_move[0] == b'w' && client_move[1] == b'i' {
                    println!("Clientul a câștigat!");
                    break;
                }
                let r = client_move[0] as usize;
                let c = client_move[1] as usize;
                board[r][c] = 2;
                // println!("Clientul a blocat poziția: {},{}", r, c);
            }
            Err(e) => {
                AppError::ReadError(e.to_string()).log();
                break;
            }
        }
    }
}

fn find_shortest_path_to_border(
    board: &[[u8; 11]; 11],
    start_x: usize,
    start_y: usize,
) -> Option<(usize, usize)> {
    let directions_even = [(-1, -1), (-1, 0), (0, -1), (0, 1), (1, -1), (1, 0)];
    let directions_odd = [(-1, 0), (-1, 1), (0, -1), (0, 1), (1, 0), (1, 1)];

    let mut queue = VecDeque::new();
    let mut visited = [[false; 11]; 11];

    queue.push_back((start_x, start_y, None));
    visited[start_x][start_y] = true;

    let distance_to_border = |x: usize, y: usize| -> i32 {
        cmp::min(
            cmp::min(x as i32, (10 - x) as i32),
            cmp::min(y as i32, (10 - y) as i32),
        )
    };

    let mut best_candidate: Option<((usize, usize), i32)> = None;

    while let Some((x, y, first_move)) = queue.pop_front() {
        if x == 0 || y == 0 || x == 10 || y == 10 {
            if let Some(fm) = first_move {
                return Some(fm);
            }
        }

        if let Some(fm) = first_move {
            let d = distance_to_border(x, y);
            if best_candidate.is_none() || d < best_candidate.unwrap().1 {
                best_candidate = Some((fm, d));
            }
        }

        let directions = if x % 2 == 0 {
            &directions_even
        } else {
            &directions_odd
        };
        for &(dx, dy) in directions {
            let nx = (x as isize + dx) as usize;
            let ny = (y as isize + dy) as usize;

            if nx < 11
                && ny < 11
                && !visited[nx][ny]
                && board[nx][ny] == 0
                && check_move(nx, ny, x as u8, y as u8, board)
            {
                visited[nx][ny] = true;
                let new_first_move = first_move.or(Some((nx, ny)));
                queue.push_back((nx, ny, new_first_move));
            }
        }
    }

    if let Some((cand, _)) = best_candidate {
        if cand != (start_x, start_y) {
            return Some(cand);
        }
    }

    let directions = if start_x % 2 == 0 {
        &directions_even
    } else {
        &directions_odd
    };
    for &(dx, dy) in directions {
        let nx = (start_x as isize + dx) as usize;
        let ny = (start_y as isize + dy) as usize;
        if nx < 11
            && ny < 11
            && board[nx][ny] == 0
            && check_move(nx, ny, start_x as u8, start_y as u8, board)
        {
            return Some((nx, ny));
        }
    }

    None
}

fn check_move(r: usize, c: usize, mouse_x: u8, mouse_y: u8, board: &[[u8; 11]; 11]) -> bool {
    if board[r][c] != 0 {
        return false;
    }

    let mut valid_moves: Vec<(i32, i32)> = Vec::new();

    let mouse_x = mouse_x as i32;
    let mouse_y = mouse_y as i32;
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

    if let Some(_pos) = valid_moves.iter().find(|&&(x, y)| x == r && y == c) {
        return true;
    }
    println!("Mutarea {}, {} este invalida!", r, c);
    false
}

fn read_ok(stream: &Option<TcpStream>) {
    let mut buffer = [0; 1024];

    if let Err(e) = stream.as_ref().unwrap().read(&mut buffer) {
        AppError::ReadError(e.to_string()).log();
    }
}

fn send_mouse_pos(slot1: &Option<TcpStream>, slot2: &Option<TcpStream>) -> (usize, usize) {
    let mut rng = rand::thread_rng();
    let row = rng.gen_range(3..=7);
    let col = rng.gen_range(3..=7);

    if let Err(e) = slot1
        .as_ref()
        .unwrap()
        .write_all(format!("{},{}", row, col).as_bytes())
    {
        AppError::WriteError(e.to_string()).log();
    }
    if let Err(e) = slot2
        .as_ref()
        .unwrap()
        .write_all(format!("{},{}", row, col).as_bytes())
    {
        AppError::WriteError(e.to_string()).log();
    }
    println!("Soarecele este la {}, {}", row, col);
    (row, col)
}

fn send_mouse_pos_computer(stream: &Option<TcpStream>) -> (usize, usize) {
    let mut rng = rand::thread_rng();
    let row = rng.gen_range(3..=7);
    let col = rng.gen_range(3..=7);

    if let Err(e) = stream
        .as_ref()
        .unwrap()
        .write_all(format!("{},{}", row, col).as_bytes())
    {
        AppError::WriteError(e.to_string()).log();
    }
    println!("Soarecele este la {}, {}", row, col);
    (row, col)
}
fn generate_wall_positions(mouse_pos: (u8, u8)) -> Vec<(u8, u8)> {
    let mut rng = rand::thread_rng();
    let mut positions = HashSet::new();

    while positions.len() < 10 {
        let row = rng.gen_range(0..11);
        let col = rng.gen_range(0..11);
        let pos = (row as u8, col as u8);

        if pos == mouse_pos {
            continue;
        }
        positions.insert(pos);
    }

    positions.into_iter().collect()
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
        if board[pos.0 as usize][pos.1 as usize] == 0 {
            ok = true;
            break;
        }
    }
    ok
}
