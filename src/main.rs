mod flow_canvas;
mod flow_grid;

use eframe::{
    App, NativeOptions,
    egui::{self, CentralPanel, Color32, TopBottomPanel, ViewportBuilder},
    icon_data, run_native,
};

const CELL_SIZE: f32 = 75.0;
const SOURCE_RADIUS: f32 = CELL_SIZE / 3.0;
const PIPE_WIDTH: f32 = CELL_SIZE * 2.0 / 7.0;
const GRID_BORDER_WIDTH: f32 = CELL_SIZE / 35.0;
const PIPE_LENGTH: f32 = (CELL_SIZE + PIPE_WIDTH) / 2.0 + GRID_BORDER_WIDTH;
const PIPE_INSET_DIST: f32 = (CELL_SIZE - PIPE_WIDTH) / 2.0 + GRID_BORDER_WIDTH;

const COLOR_INDEX: [(&str, Color32); 9] = [
    ("Red", Color32::from_rgb(255, 0, 0)),
    ("Green", Color32::from_rgb(0, 200, 0)),
    ("Blue", Color32::from_rgb(0, 0, 255)),
    ("Yellow", Color32::from_rgb(255, 255, 0)),
    ("Orange", Color32::from_rgb(255, 165, 0)),
    ("Purple", Color32::from_rgb(128, 0, 128)),
    ("Cyan", Color32::from_rgb(0, 255, 255)),
    ("Pink", Color32::from_rgb(255, 192, 203)),
    ("Dark Red", Color32::from_rgb(128, 0, 0)),
];

struct FlowSolverApp {
    flow_canvas: flow_canvas::FlowCanvas,
}

impl FlowSolverApp {
    pub fn with_size(width: usize, height: usize) -> Self {
        FlowSolverApp {
            flow_canvas: flow_canvas::FlowCanvas::with_size(width, height),
        }
    }
}

impl App for FlowSolverApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Flow Solver");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Quit").clicked() {
                        let ctx = ctx.clone();
                        std::thread::spawn(move || {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        });
                    }
                });
            });
        });
        CentralPanel::default().show(ctx, |ui| {
            ui.label("Click on the grid to place a flow source. Click and drag to connect them.");

            ui.add(&mut self.flow_canvas);
            ui.horizontal(|ui| {
                ui.label(format!(
                    "Next color: {}",
                    COLOR_INDEX
                        .get(self.flow_canvas.grid.next_color())
                        .unwrap_or(&("(No Defined color)", Color32::BLACK))
                        .0,
                ));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.button("toggle sources locked").clicked().then(|| {
                        self.flow_canvas.can_edit_sources = !self.flow_canvas.can_edit_sources;
                    });
                });
            });
            ui.button("Reset")
                .on_hover_text("Reset the grid to its initial state")
                .clicked()
                .then(|| {
                    self.flow_canvas = flow_canvas::FlowCanvas::with_size(
                        self.flow_canvas.grid.width,
                        self.flow_canvas.grid.height,
                    );
                });
        });
    }
}
fn main() -> eframe::Result {
    const GRID_HEIGHT: usize = 7;
    const GRID_WIDTH: usize = 7;

    let ui_width = GRID_WIDTH as f32 * CELL_SIZE + 55.0;
    let ui_height = GRID_HEIGHT as f32 * CELL_SIZE + 130.0;

    let native_options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([ui_width, ui_height])
            .with_min_inner_size([ui_width, ui_height])
            .with_icon(
                icon_data::from_png_bytes(&include_bytes!("../assets/pipe-512.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };

    run_native(
        "Flow Solver",
        native_options,
        Box::new(|_cc| Ok(Box::new(FlowSolverApp::with_size(GRID_WIDTH, GRID_HEIGHT)))),
    )
}
