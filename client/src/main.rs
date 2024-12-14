use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

fn choose_option() -> i32 {
    use std::io::{self, Write};
    use rand::Rng;

    println!("Doresti sa fii repartizat intr-o camera random? : 1");
    println!("Doresti sa creezi o camera? : 2");
    println!("Doresti sa intri intr-o camera deja repartizata? : 3");

    let mut input = String::new();
    io::stdout().flush().expect("Eroare la flush");
    io::stdin().read_line(&mut input).expect("Eroare la citirea de la tastatură");

    let input = input.trim();
    let mut res = 0;

    match input.parse::<i32>() {
        Ok(num) => {
            if num == 1 {
                res = -1;
            } else if num == 2 {
                let mut rng = rand::rng();
                let random_number = rng.random_range(1..=100);
                res = random_number;
            } else if num == 3 {
                println!("Introduceti codul camerei:");
                let mut input = String::new();
                io::stdout().flush().expect("Eroare la flush");
                io::stdin().read_line(&mut input).expect("Eroare la citirea de la tastatură");
                let input = input.trim();

                match input.parse::<i32>() {
                    Ok(num) => res = num,
                    Err(e) => println!("Eroare la parsare: {}", e),
                }
            }
        }
        Err(e) => println!("Eroare la parsare: {}", e),
    }
    res
}
fn main() -> std::io::Result<()> {
    let server_address = "127.0.0.1:8080";
    let mut stream = TcpStream::connect(server_address)?;

    println!("Conectat la serverul {}", server_address);

    let res = choose_option();

    println!("Codul camerei este {res} ");
    stream.write_all(res.to_string().as_bytes()).expect("Eroare la scriere");
    stream.flush().expect("Eroare la flush");

    let mut buffer = vec![0; 1024];

    loop {

        let n = stream.read(&mut buffer)?;
        if n == 0 {
            println!("Serverul a închis conexiunea.");
            break;
        }
        let response = String::from_utf8_lossy(&buffer[..n]);
        println!("Mesaj primit de la server: {}", response);
    }

    thread::sleep(Duration::from_secs(1000));

    Ok(())
}
