use common::{AppError, Position};
use eframe::egui::{self, Pos2, Rect, RichText, Shape, Stroke, TextureOptions, Vec2};
use image::{load_from_memory_with_format, ImageFormat};
use std::sync::{Arc, RwLock};
mod game;
use game::{check_move, send_move, Game, Screen, State};

struct GameApp {
    game: Arc<RwLock<Game>>,
}

impl eframe::App for GameApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut game = self.game.write().unwrap();

        match game.screen {
            Screen::MainMenu => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ctx.set_visuals(egui::Visuals::light());
                    ui.vertical_centered(|ui| {
                        ui.add_space(50.0);
                        ui.label(RichText::new("Trap the mouse!").size(32.0));
                        ui.add_space(30.0);

                        ui.group(|ui| {
                            ui.set_max_width(300.0);
                            ui.set_min_width(300.0);
                            ui.vertical_centered(|ui| {
                                let button_size = Vec2::new(300.0, 50.0);

                                if ui
                                    .add_sized(
                                        button_size,
                                        egui::Button::new(
                                            RichText::new("Camera Random").size(18.0),
                                        ),
                                    )
                                    .clicked()
                                {
                                    println!("Camera Random selectata");
                                    game.try_connect = true;
                                    game.state = State::Connect;
                                }

                                ui.add_space(10.0);

                                if ui
                                    .add_sized(
                                        button_size,
                                        egui::Button::new(
                                            RichText::new("Creaza Camera Noua").size(18.0),
                                        ),
                                    )
                                    .clicked()
                                {
                                    println!("Creaza Camera Noua selectata");
                                    game.show_input_1 = true;
                                    game.show_input_2 = false;
                                }

                                if game.show_input_1 {
                                    ui.group(|ui| {
                                        ui.vertical_centered(|ui| {
                                            ui.label(RichText::new("Numar camera:").heading());
                                            ui.add_sized(
                                                Vec2::new(200.0, 30.0),
                                                egui::TextEdit::singleline(&mut game.room_input)
                                                    .font(egui::TextStyle::Heading),
                                            );
                                            ui.add_space(10.0);
                                            if ui
                                                .add_sized(
                                                    Vec2::new(280.0, 50.0),
                                                    egui::Button::new(
                                                        RichText::new("Submit").size(18.0),
                                                    ),
                                                )
                                                .clicked()
                                            {
                                                println!(
                                                    "Intrat in camera cu numarul: {}",
                                                    game.room_input
                                                );
                                                game.show_input_1 = false;
                                                game.try_connect = true;
                                                game.state = State::Connect;
                                            }
                                        });
                                    });
                                }

                                ui.add_space(10.0);

                                if ui
                                    .add_sized(
                                        button_size,
                                        egui::Button::new(
                                            RichText::new("Intra intr-o Camera Existenta")
                                                .size(18.0),
                                        ),
                                    )
                                    .clicked()
                                {
                                    game.show_input_2 = true;
                                    game.show_input_1 = false;
                                }

                                if game.show_input_2 {
                                    ui.add_space(20.0);
                                    ui.group(|ui| {
                                        ui.vertical_centered(|ui| {
                                            ui.label(RichText::new("Numar camera:").heading());
                                            ui.add_sized(
                                                Vec2::new(200.0, 30.0),
                                                egui::TextEdit::singleline(&mut game.room_input)
                                                    .font(egui::TextStyle::Heading),
                                            );
                                            ui.add_space(10.0);
                                            if ui
                                                .add_sized(
                                                    Vec2::new(280.0, 50.0),
                                                    egui::Button::new(
                                                        RichText::new("Submit").size(18.0),
                                                    ),
                                                )
                                                .clicked()
                                            {
                                                println!(
                                                    "Intrat in camera cu numarul: {}",
                                                    game.room_input
                                                );
                                                game.show_input_2 = false;
                                                game.try_connect = true;
                                                game.state = State::Connect;
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
                                let button_size = Vec2::new(300.0, 50.0);
                                if ui
                                    .add_sized(
                                        button_size,
                                        egui::Button::new(
                                            RichText::new("Joaca cu Calculatorul").size(18.0),
                                        ),
                                    )
                                    .clicked()
                                {
                                    println!("Joaca cu Calculatorul selectata");
                                    game.room_input = String::from("-2");
                                    game.try_connect = true;
                                }
                            });
                        });

                        ui.add_space(50.0);
                    });
                });
            }
            Screen::WaitingForPlayers => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new(format!(
                                "Asteptati jucatori in camera {}...",
                                game.room_input
                            ))
                            .size(24.0),
                        );
                    });
                });
            }
            Screen::ConnectingError => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("Eroare la conectare!").size(24.0));
                    });
                });
            }
            Screen::GameBoard => {
                if game.mouse_texture.is_none() {
                    let image_data = include_bytes!("mouse.png");

                    let image = load_from_memory_with_format(image_data, ImageFormat::Png)
                        .expect("Failed to load image");
                    let image = image.to_rgba8();

                    let size = [image.width() as usize, image.height() as usize];

                    let pixels: Vec<_> = image.pixels().flat_map(|p| p.0).collect();

                    let hexagon_texture = ctx.load_texture(
                        "mouse",
                        egui::ColorImage::from_rgba_unmultiplied(size, &pixels),
                        TextureOptions::default(),
                    );
                    game.mouse_texture = Some(hexagon_texture);
                }

                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(30.0);
                        ui.label(RichText::new("Jocul a inceput...").size(24.0));

                        if game.player_text.is_some() {
                            ui.label(RichText::new(game.player_text.as_ref().unwrap()).size(18.0));
                        }

                        if game.has_to_read && !game.win {
                            ui.label(RichText::new("Asteptati tura..").size(18.0));
                        }
                    });

                    ui.vertical(|ui| {
                        ui.add_space(10.0);

                        let hex_radius = 23.0;
                        let hex_height = 2.0 * hex_radius;
                        let hex_width = (3.0_f32.sqrt()) * hex_radius;
                        let horizontal_spacing = hex_width * 1.10;
                        let vertical_spacing = hex_height * 0.85;

                        let painter = ui.painter();
                        let start_x = 160.0;
                        let start_y = 140.0;

                        for row in 0..11 {
                            for col in 0..11 {
                                let x = start_x + col as f32 * horizontal_spacing;
                                let y = start_y + row as f32 * vertical_spacing;

                                let x = if row % 2 == 1 { x + hex_width * 0.5 } else { x };

                                let hex_points = hexagon_points(x, y, hex_radius);

                                if game.board[row][col] == 1 {
                                    if let Some(texture) = &game.mouse_texture {
                                        let rect = Rect::from_center_size(
                                            Pos2::new(x, y),
                                            Vec2::new(hex_radius * 1.5, hex_radius * 1.5),
                                        );

                                        painter.image(
                                            texture.id(),
                                            rect,
                                            Rect::from_min_max(
                                                Pos2::new(0.0, 0.0),
                                                Pos2::new(1.0, 1.0),
                                            ),
                                            egui::Color32::WHITE,
                                        );

                                        painter.add(Shape::convex_polygon(
                                            hex_points.clone(),
                                            egui::Color32::TRANSPARENT,
                                            Stroke::new(2.0, egui::Color32::BLACK),
                                        ));
                                    }
                                } else {
                                    let color = if game.board[row][col] == 0 {
                                        egui::Color32::from_rgb(100, 200, 100)
                                    } else {
                                        egui::Color32::from_rgb(255, 51, 0)
                                    };

                                    painter.add(Shape::convex_polygon(
                                        hex_points.clone(),
                                        color,
                                        Stroke::new(2.0, egui::Color32::BLACK),
                                    ));
                                }

                                let response = ui.interact(
                                    egui::Rect::from_center_size(
                                        egui::Pos2::new(x, y),
                                        egui::Vec2::new(hex_width, hex_height),
                                    ),
                                    egui::Id::new(format!("hexagon_{}_{}", row, col)),
                                    egui::Sense::click(),
                                );

                                if response.clicked()
                                    && !game.has_to_read
                                    && check_move(
                                        row,
                                        col,
                                        &game.mouse,
                                        game.player as u8,
                                        &game.board,
                                    )
                                    && game.board[row][col] == 0
                                {
                                    send_move(&mut game, row, col);
                                    game.has_to_read = true;
                                }
                            }
                        }
                    });
                    if game.win {
                        let screen_rect = ctx.screen_rect();
                        let painter = ctx.layer_painter(egui::LayerId::new(
                            egui::Order::Foreground,
                            "overlay".into(),
                        ));

                        painter.rect_filled(
                            screen_rect,
                            0.0,
                            egui::Color32::from_rgba_premultiplied(0, 0, 0, 200),
                        );

                        painter.text(
                            screen_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            &game.win_state,
                            egui::FontId::proportional(50.0),
                            egui::Color32::from_rgba_premultiplied(255, 255, 255, 200),
                        );

                        egui::Area::new("win_menu_button")
                            .order(egui::Order::Foreground)
                            .anchor(egui::Align2::CENTER_CENTER, [0.0, 100.0])
                            .show(ctx, |ui| {
                                let button_size = egui::Vec2::new(300.0, 50.0);

                                if ui
                                    .add_sized(
                                        button_size,
                                        egui::Button::new(
                                            egui::RichText::new("Play Again").size(18.0),
                                        ),
                                    )
                                    .clicked()
                                {
                                    println!("Play Again!");

                                    game.screen = Screen::MainMenu;
                                    game.show_input_1 = false;
                                    game.show_input_2 = false;
                                    game.room_input = String::from("-1");
                                    game.player_text = None;
                                    game.player = -1;
                                    game.initial_mouse = false;
                                    game.mouse = Position::new(0, 0);
                                    game.board = [[0; 11]; 11];
                                    game.has_to_read = false;
                                    game.win = false;
                                    game.state = State::PlayAgain;
                                    game.win_state = String::new();
                                    game.try_connect = false;
                                }
                            });
                    }
                });
            }
        }
        ctx.request_repaint();
    }
}

fn hexagon_points(center_x: f32, center_y: f32, radius: f32) -> Vec<Pos2> {
    (0..6)
        .map(|i| {
            let angle = std::f32::consts::PI / 3.0 * i as f32 - std::f32::consts::PI / 6.0;
            Pos2::new(
                center_x + radius * angle.cos(),
                center_y + radius * angle.sin(),
            )
        })
        .collect()
}

fn main() {
    std::env::set_var("WINIT_UNIX_BACKEND", "x11");

    let options = eframe::NativeOptions::default();
    let game = Arc::new(RwLock::new(Game::default()));
    Game::start_tcp(Arc::clone(&game));
    match eframe::run_native(
        "Trap the mouse!",
        options,
        Box::new(|_cc| Box::new(GameApp { game })),
    ) {
        Ok(_) => {}
        Err(e) => {
            AppError::GraphicsError(e.to_string()).log();
        }
    }
}
