use rand::Rng;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::thread;

struct Room {
    slot1: Option<TcpStream>,
    slot2: Option<TcpStream>,
    is_full: bool,
    code: i32,
    is_taken: bool,
}

impl Room {
    fn new(code: i32) -> Self {
        Room {
            slot1: None,
            slot2: None,
            is_full: false,
            code,
            is_taken: false,
        }
    }

    fn add_to_room(&mut self, stream: &TcpStream) -> bool {
        if self.slot1.is_none() {
            self.slot1 = Some(stream.try_clone().unwrap());
            if self.slot2.is_some() {
                self.is_full = true;
            }
            return true;
        }
        if self.slot2.is_none() {
            self.slot2 = Some(stream.try_clone().unwrap());
            if self.slot1.is_some() {
                self.is_full = true;
            }
            return true;
        }
        false
    }

    fn remove_from_room(&mut self, stream: &TcpStream) -> bool {
        if let Some(ref s1) = self.slot1 {
            if stream.peer_addr().unwrap() == s1.peer_addr().unwrap() {
                self.slot1 = None;
                self.is_full = false;
                return true;
            }
        }
        if let Some(ref s2) = self.slot2 {
            if stream.peer_addr().unwrap() == s2.peer_addr().unwrap() {
                self.is_full = false;
                return true;
            }
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

    fn add_to_room(&mut self, stream: TcpStream) -> i32 {
        for room in self.rooms.iter_mut() {
            if room.slot1.is_none() || room.slot2.is_none() {
                room.add_to_room(&stream);
                return room.code;
            }
        }

        self.rooms.push(Room::new(self.rooms.len() as i32));
        let room_id = self.rooms.len() - 1;
        self.rooms[room_id].add_to_room(&stream);
        room_id as i32
    }
    fn add_to_spec_room(&mut self, stream: TcpStream, code: i32) -> i32 {
        for room in self.rooms.iter_mut() {
            if room.code == code && (room.slot1.is_none() || room.slot2.is_none()) {
                room.add_to_room(&stream);
                return room.code;
            }
        }
        -1
    }
    fn add_to_new_room(&mut self, stream: &TcpStream, code: i32) -> i32 {
        self.rooms.push(Room::new(code));
        let room_id = self.rooms.len() - 1;
        self.rooms[room_id].add_to_room(stream);
        code
    }
    fn remove_from_room(&mut self, stream: &TcpStream) {
        for room in &mut self.rooms {
            room.remove_from_room(stream);
        }
    }

    fn show_rooms(&self) {
        println!("Camerele curente:");
        for room in &self.rooms {
            println!("{:?} {:?}", room.slot1, room.slot2);
        }
    }
    fn check_room(&mut self, room_id: i32) -> bool {
        for room in &self.rooms {
            if room.code == room_id && room.is_full {
                return true;
            }
        }
        false
    }
}

fn handle_client_1(mut stream: TcpStream, rooms_manager: Arc<RwLock<RoomsManager>>) {
    let mut buffer: [u8; 100] = [0; 100];
    let mut room: i32 = -1;
    let mut rm_id: i32 = -1;

    match stream.read(&mut buffer) {
        Ok(n) if n > 0 => {
            let message = String::from_utf8_lossy(&buffer[..n]);
            let message = message.trim();

            match message.parse::<i32>() {
                Ok(num) => {
                    println!("Cod camera de la client: {}", num);
                    room = num;
                }
                Err(e) => println!("Eroare la parsare: {}", e),
            }
        }
        Ok(0) => {
            println!(
                "Clientul {} s-a deconectat inainte sa fie repartizat intr-o camera.",
                stream.peer_addr().unwrap()
            );
        }
        Err(e) => {
            eprintln!("Eroare la citirea datelor de la client: {}", e);
        }
        _ => {}
    }
    match room {
        -1 => {
            let mut rooms_manager = rooms_manager.write().unwrap();
            rm_id = rooms_manager.add_to_room(stream.try_clone().unwrap());
            println!("Clientul a fost adăugat în camera {}", rm_id);

            stream
                .write_all(rm_id.to_string().as_bytes())
                .expect("Eroare la scriere");

            rooms_manager.show_rooms();
        }
        _ => {
            let mut rooms_manager = rooms_manager.write().unwrap();
            rm_id = rooms_manager.add_to_spec_room(stream.try_clone().unwrap(), room);

            if rm_id == -1 {
                rm_id = rooms_manager.add_to_new_room(&stream, room);
            }

            println!("Clientul a fost adăugat în camera {}", rm_id);
            stream
                .write_all(rm_id.to_string().as_bytes())
                .expect("Eroare la scriere");
            rooms_manager.show_rooms();
        }
    }
    loop {
        {
            let mut rooms_manager = rooms_manager.write().unwrap();
            if rooms_manager.check_room(rm_id) {
                break;
            }
        }
    }
    let message = format!("Jocul a inceput in camera! {}", rm_id);
    if let Err(e) = stream.write_all(message.as_bytes()) {
        eprintln!("Eroare la scriere către client: {}", e);
    }
    std::thread::sleep(std::time::Duration::from_secs(2));
    handle_room(rm_id, rooms_manager);
}

fn handle_room(room_id: i32, rooms_manager: Arc<RwLock<RoomsManager>>) {
    let mut slot1_: Option<TcpStream> = None;
    let mut slot2_: Option<TcpStream> = None;

    {
        let mut rooms_manager = rooms_manager.write().unwrap();
        for room in &mut rooms_manager.rooms {
            if room.code == room_id {
                if room.is_taken == true {
                    return;
                }
                room.is_taken = true;
                slot1_ = room.slot1.as_ref().and_then(|s| s.try_clone().ok());
                slot2_ = room.slot2.as_ref().and_then(|s| s.try_clone().ok());

                break;
            }
        }
    }

    println!("<----------{:?} {:?}----------->", slot1_, slot2_);

    let mut rng = rand::thread_rng();
    let first = rng.gen_range(1..=2);
    println!("First {first}");
    slot1_
        .unwrap()
        .write_all(first.to_string().as_bytes())
        .expect("Eroare la scriere!!!");
    let temp;
    if first == 1 {
        temp = 2;
    } else {
        temp = 1;
    }
    slot2_
        .unwrap()
        .write_all(temp.to_string().as_bytes())
        .expect("Eroare la scriere!!!");
    println!("Second {temp}");
    loop {}
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("Serverul rulează pe 127.0.0.1:8080");

    let rooms_manager = Arc::new(RwLock::new(RoomsManager::new()));

    for stream in listener.incoming() {
        let rooms_manager = Arc::clone(&rooms_manager);
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client_1(stream, rooms_manager.clone());
                });
            }
            Err(e) => {
                eprintln!("Eroare la acceptarea conexiunii: {}", e);
            }
        }
    }

    Ok(())
}
