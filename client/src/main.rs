use crate::egui::Pos2;
use crate::egui::Response;
use crate::egui::Sense;
use crate::egui::Shape;
use crate::egui::Stroke;
use crate::egui::Ui;
use eframe::egui::{self, RichText};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;
fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native("Meniu", options, Box::new(|_cc| Box::new(Game::default())))
}

enum AppState {
    MainMenu,
    WaitingForPlayers,
    ConnectingError,
    GameBoard,
}

struct Game {
    room_selection: String,
    show_room_input_1: bool,
    show_room_input_2: bool,
    state: AppState,
    wait: i32,
    wait_bool: bool,
    stream: Option<TcpStream>,
    player: i32,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            room_selection: String::from("-1"),
            show_room_input_1: false,
            show_room_input_2: false,
            state: AppState::MainMenu,
            wait: -1,
            wait_bool: false,
            stream: None,
            player: -1,
        }
    }
}

impl eframe::App for Game {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::light());

        match self.state {
            AppState::MainMenu => self.show_main_menu(ctx),
            AppState::WaitingForPlayers => self.show_waiting_for_players(ctx),
            AppState::ConnectingError => self.connecting_error(ctx),
            AppState::GameBoard => self.show_game_board(ctx),
        }
    }
}

impl Game {
    fn show_main_menu(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.set_visuals(egui::Visuals::light());
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label(egui::RichText::new("Trap the mouse").size(32.0));
                ui.add_space(30.0);

                ui.group(|ui| {
                    ui.set_max_width(300.0);
                    ui.set_min_width(300.0);
                    ui.vertical_centered(|ui| {
                        let button_size = egui::Vec2::new(300.0, 50.0);

                        if ui
                            .add_sized(
                                button_size,
                                egui::Button::new(egui::RichText::new("Camera Random").size(18.0)),
                            )
                            .clicked()
                        {
                            // Adauga functionalitatea pentru Camera Random aici
                            println!("Camera Random selectata");
                            self.state = AppState::WaitingForPlayers;
                            self.connect_to_server(ctx);
                        }

                        ui.add_space(10.0);

                        if ui
                            .add_sized(
                                button_size,
                                egui::Button::new(
                                    egui::RichText::new("Creaza Camera Noua").size(18.0),
                                ),
                            )
                            .clicked()
                        {
                            println!("Creaza Camera Noua selectata");
                            self.show_room_input_1 = true;
                        }

                        if self.show_room_input_1 {
                            ui.group(|ui| {
                                ui.vertical_centered(|ui| {
                                    ui.label(egui::RichText::new("Numar camera:").heading());
                                    ui.add_sized(
                                        egui::Vec2::new(200.0, 30.0),
                                        egui::TextEdit::singleline(&mut self.room_selection)
                                            .font(egui::TextStyle::Heading),
                                    );
                                    ui.add_space(10.0);
                                    if ui
                                        .add_sized(
                                            egui::Vec2::new(280.0, 50.0),
                                            egui::Button::new(
                                                egui::RichText::new("Submit").size(18.0),
                                            ),
                                        )
                                        .clicked()
                                    {
                                        println!(
                                            "Intrat in camera cu numarul: {}",
                                            self.room_selection
                                        );
                                        self.show_room_input_1 = false; // Ascunde input-ul dupa submit
                                        self.state = AppState::WaitingForPlayers;
                                        self.connect_to_server(ctx);
                                    }
                                });
                            });
                        }

                        ui.add_space(10.0);

                        if ui
                            .add_sized(
                                button_size,
                                egui::Button::new(
                                    egui::RichText::new("Intra intr-o Camera Existenta").size(18.0),
                                ),
                            )
                            .clicked()
                        {
                            self.show_room_input_2 = true;
                        }

                        if self.show_room_input_2 {
                            ui.add_space(20.0);
                            ui.group(|ui| {
                                ui.vertical_centered(|ui| {
                                    ui.label(egui::RichText::new("Numar camera:").heading());
                                    ui.add_sized(
                                        egui::Vec2::new(200.0, 30.0),
                                        egui::TextEdit::singleline(&mut self.room_selection)
                                            .font(egui::TextStyle::Heading),
                                    );
                                    ui.add_space(10.0);
                                    if ui
                                        .add_sized(
                                            egui::Vec2::new(280.0, 50.0),
                                            egui::Button::new(
                                                egui::RichText::new("Submit").size(18.0),
                                            ),
                                        )
                                        .clicked()
                                    {
                                        // Adauga functionalitatea pentru Intra in Camera
                                        println!(
                                            "Intrat in camera cu numarul: {}",
                                            self.room_selection
                                        );
                                        self.show_room_input_2 = false; // Ascunde input-ul dupa submit
                                        self.state = AppState::WaitingForPlayers;
                                        self.connect_to_server(ctx);
                                    }
                                });
                            });
                        }
                    });
                });

                ui.add_space(50.0);
                ui.separator();
                ui.add_space(20.0);

                ui.group(|ui| {
                    ui.vertical_centered(|ui| {
                        let button_size = egui::Vec2::new(300.0, 50.0);
                        if ui
                            .add_sized(
                                button_size,
                                egui::Button::new(
                                    egui::RichText::new("Joaca cu Calculatorul").size(18.0),
                                ),
                            )
                            .clicked()
                        {
                            // Adauga functionalitatea pentru Joaca cu Calculatorul aici
                            println!("Joaca cu Calculatorul selectata");
                        }
                    });
                });

                ui.add_space(50.0);
            });
        });
    }

    fn show_waiting_for_players(&mut self, ctx: &egui::Context) {
        self.wait_bool = true;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label(RichText::new("Asteptati pana se conecteaza alti playeri...").size(24.0));
                ui.label(RichText::new(format!("Camera {}", self.room_selection)).size(18.0));
            });
            if self.wait_bool {
                self.wait += 1;
            }
            if self.wait == 10 {
                ctx.request_repaint();

                println!("Macar aici ?");
                let mut buffer = vec![0; 1024];
                let _n = self.stream.as_ref().unwrap().read(&mut buffer);

                println!("START! {}", String::from_utf8_lossy(&buffer[.._n.unwrap()]));
                self.state = AppState::GameBoard;
                self.wait_bool = false;
            }
        });
    }

    fn connecting_error(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label(RichText::new("Eroare la conectarea la server...").size(24.0));
            });
        });
    }
    fn draw_hexagon_grid(&mut self, ui: &mut egui::Ui) {
        let hex_radius = 25.0; // Dimensiunea hexagonului (jumătate din lățime)
        let hex_width = hex_radius * 2.0;
        let hex_height = hex_radius * (3.0_f32.sqrt());
        let cols = 10;
        let rows = 10;

        for row in 0..rows {
            for col in 0..cols {
                let x_offset = col as f32 * hex_width * 0.75;
                let y_offset = row as f32 * hex_height;

                let x = x_offset + if row % 2 == 1 { hex_width * 0.375 } else { 0.0 };
                let y = y_offset;

                let center = egui::Pos2::new(x + hex_radius, y + hex_height / 2.0);

                // Calculează vârfurile hexagonului
                let mut points = vec![];
                for i in 0..6 {
                    let angle = std::f32::consts::PI / 3.0 * i as f32;
                    points.push(egui::Pos2::new(
                        center.x + hex_radius * angle.cos(),
                        center.y + hex_radius * angle.sin(),
                    ));
                }

                // Creează un PathShape pentru hexagon
                let path = egui::epaint::PathShape {
                    points,
                    closed: true,
                    fill: egui::Color32::from_rgb(100, 150, 200),
                    stroke: egui::Stroke::new(2.0, egui::Color32::WHITE),
                };

                // Adaugă path-ul pe canvas
                ui.painter().add(egui::epaint::Shape::Path(path));
            }
        }
    }

    fn show_game_board(&mut self, ctx: &egui::Context) {
        if self.player == -1 {
            let mut buffer = [0; 1024];
            // self.stream
            //     .as_ref()
            //     .unwrap()
            //     .set_read_timeout(Some(Duration::from_secs(5)))
            //     .unwrap();
            let result = self.stream.as_ref().unwrap().read(&mut buffer);
            match result {
                Ok(n) if n > 0 => {
                    let response = String::from_utf8_lossy(&buffer[..n]);
                    println!("Răspuns de la server: {}", response);
                    self.player = convert_to_i32(response.to_string());
                }
                Ok(_) => println!("Conexiunea închisă."),
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => println!("Timeout!"),
                Err(e) => println!("Eroare la citire: {}", e),
            }
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label(RichText::new("Jocul a inceput...").size(24.0));
                ui.label(RichText::new(format!("Esti player-ul {}", self.player)).size(18.0));
            });
            ui.vertical_centered(|ui| {
                let start = ui.available_rect_before_wrap().min + egui::Vec2::new(20.0, 20.0);
                let hex_size = 20.0; // Dimensiunea hexagoanelor
                self.draw_hexagon_grid(ui, start, hex_size, 11, 11);
                // ui.vertical_centered(|ui| {
                //     let button_size = egui::Vec2::new(40.0, 40.0);
                //     let rows = 11;
                //     let cols = 11;

                //     for row in 0..rows {
                //         ui.horizontal(|ui| {
                //             if row % 2 == 0 {
                //                 ui.add_space(130.0);
                //             } else {
                //                 ui.add_space(150.0);
                //             }
                //             for col in 0..cols {
                //                 // let button_label = format!("({}, {})", row + 1, col + 1);
                //                 if ui.add_sized(button_size, egui::Button::new("")).clicked() {
                //                     println!(
                //                         "Butonul de la ({}, {}) a fost apasat.",
                //                         row + 1,
                //                         col + 1
                //                     );
                //                 }
                //             }
                //         });
                //     }
                // });
            });
        });
    }
    fn draw_hexagon_button(
        &mut self,
        ui: &mut Ui,
        center: Pos2,
        size: f32,
        label: &str,
    ) -> Response {
        let mut points = vec![];
        for i in 0..6 {
            let angle = i as f32 * std::f32::consts::PI / 3.0;
            points.push(Pos2::new(
                center.x + size * angle.cos(),
                center.y + size * angle.sin(),
            ));
        }

        let hexagon = Shape::convex_polygon(
            points.clone(),
            ui.visuals().widgets.inactive.bg_fill,
            Stroke::new(2.0, ui.visuals().widgets.inactive.bg_stroke.color),
        );

        let rect = egui::Rect::from_min_max(
            points.iter().fold(points[0], |min, &p| Pos2::min(min, p)),
            points.iter().fold(points[0], |max, &p| Pos2::max(max, p)),
        );

        let painter = ui.painter_at(rect);
        painter.add(hexagon);

        let response = ui.allocate_rect(rect, Sense::click());
        if response.clicked() {
            painter.add(Shape::convex_polygon(
                points,
                ui.visuals().widgets.active.bg_fill,
                Stroke::new(2.0, ui.visuals().widgets.active.bg_stroke.color),
            ));
        }

        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            label,
            egui::TextStyle::Button.resolve(ui.style()),
            ui.visuals().widgets.inactive.fg_stroke.color,
        );

        response
    }

    fn draw_hex_grid(&mut self, ui: &mut Ui, start: Pos2, hex_size: f32, rows: usize, cols: usize) {
        let hex_width = hex_size * 2.0;
        let hex_height = hex_size * 3f32.sqrt(); // Înălțimea hexagonului
        let y_offset = hex_height * 0.75; // Suprapunerea rândurilor
        let x_offset = hex_width * 0.5; // Decalajul între rânduri

        for row in 0..rows {
            for col in 0..cols {
                let x =
                    start.x + col as f32 * hex_width + if row % 2 == 1 { x_offset } else { 0.0 };
                let y = start.y + row as f32 * y_offset;

                let center = Pos2::new(x, y);
                let label = format!("{},{}", row, col);

                if self
                    .draw_hexagon_button(ui, center, hex_size, &label)
                    .clicked()
                {
                    println!("Hexagon clicked: {}", label);
                }
            }
        }
    }
    fn connect_to_server(&mut self, ctx: &egui::Context) {
        match TcpStream::connect("127.0.0.1:8080") {
            Ok(mut stream) => {
                let room_id = convert_to_i32(self.room_selection.clone());

                stream
                    .write_all(room_id.to_string().as_bytes())
                    .expect("Eroare la scriere");
                stream.flush().expect("Eroare la flush");

                let mut buffer = vec![0; 1024];

                let n = stream.read(&mut buffer);
                let n = n.unwrap();
                if n == 0 {
                    println!("Serverul a închis conexiunea.");
                    return;
                }
                let response = String::from_utf8_lossy(&buffer[..n]);

                self.room_selection = response.to_string();
                self.stream = Some(stream);
            }

            Err(e) => {
                eprintln!("Eroare la conectarea la server: {}", e);
                self.state = AppState::ConnectingError;
            }
        }
    }
}

fn convert_to_i32(s: String) -> i32 {
    match s.parse::<i32>() {
        Ok(num) => num,
        Err(e) => {
            println!("Eroare la parsare: {}", e);
            -1
        }
    }
}
