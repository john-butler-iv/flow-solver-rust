/// This file handles most of the complexity with the UI. That is, this file is responsible for
/// 1. taking user input and interpretting it as commands for the underlying data model in flow_grid
/// 2. interpretting the data from flow_grid and displaying it to the user
use crate::{
    CELL_SIZE, COLOR_INDEX, GRID_BORDER_WIDTH, PIPE_INSET_DIST, PIPE_LENGTH, PIPE_WIDTH,
    SOURCE_RADIUS,
    flow_grid::{self, CellColor, Direction},
};
use eframe::egui::{
    self, Color32, Context, CornerRadius, Painter, Pos2, Rect, Response, Sense, Vec2, Widget,
};

pub struct FlowCanvas {
    pub grid: flow_grid::FlowGrid,
    have_laid_pipe: bool,
    previous_row_col: Option<(usize, usize)>,
    pub can_edit_sources: bool,
}

impl Widget for &mut FlowCanvas {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (canvas_rect, response) = ui.allocate_exact_size(
            Vec2::new(
                GRID_BORDER_WIDTH + (CELL_SIZE + GRID_BORDER_WIDTH) * self.grid.width as f32,
                GRID_BORDER_WIDTH + (CELL_SIZE + GRID_BORDER_WIDTH) * self.grid.height as f32,
            ),
            Sense::click_and_drag(),
        );

        let painter = ui.painter_at(canvas_rect);

        self.draw_grid_lines(&painter, &canvas_rect, ui.visuals().window_stroke().color);

        for row in 0..self.grid.height {
            for col in 0..self.grid.width {
                // TODO maybe could be better to get an iterator from grid? idk.
                let x0 = col as f32 * (CELL_SIZE + GRID_BORDER_WIDTH)
                    + canvas_rect.min.x
                    + GRID_BORDER_WIDTH;
                let y0 = row as f32 * (CELL_SIZE + GRID_BORDER_WIDTH)
                    + canvas_rect.min.y
                    + GRID_BORDER_WIDTH;
                let cell = self.grid.get(row, col).expect("looping in bounds");

                let color = interpret_cell_color(cell.color);

                if cell.is_source {
                    painter.circle_filled(
                        Pos2::from([x0 + CELL_SIZE / 2.0, y0 + CELL_SIZE / 2.0]),
                        SOURCE_RADIUS,
                        color,
                    );
                }
                if cell.is_connected_up {
                    painter.rect_filled(
                        Rect::from_min_size(
                            Pos2::from([x0 + PIPE_INSET_DIST, y0]),
                            Vec2::from([PIPE_WIDTH, PIPE_LENGTH]),
                        ),
                        CornerRadius {
                            ne: 0,
                            nw: 0,
                            se: PIPE_WIDTH as u8 / 2,
                            sw: PIPE_WIDTH as u8 / 2,
                        },
                        color,
                    );
                }
                if cell.is_connected_down {
                    painter.rect_filled(
                        Rect::from_min_size(
                            Pos2::from([x0 + PIPE_INSET_DIST, y0 + PIPE_INSET_DIST]),
                            Vec2::from([PIPE_WIDTH, PIPE_LENGTH]),
                        ),
                        CornerRadius {
                            ne: PIPE_WIDTH as u8 / 2,
                            nw: PIPE_WIDTH as u8 / 2,
                            se: 0,
                            sw: 0,
                        },
                        color,
                    );
                }
                if cell.is_connected_left {
                    painter.rect_filled(
                        Rect::from_min_size(
                            Pos2::from([x0, y0 + PIPE_INSET_DIST]),
                            Vec2::from([PIPE_LENGTH, PIPE_WIDTH]),
                        ),
                        CornerRadius {
                            ne: PIPE_WIDTH as u8 / 2,
                            nw: 0,
                            se: PIPE_WIDTH as u8 / 2,
                            sw: 0,
                        },
                        color,
                    );
                }
                if cell.is_connected_right {
                    painter.rect_filled(
                        Rect::from_min_size(
                            Pos2::from([x0 + PIPE_INSET_DIST, y0 + PIPE_INSET_DIST]),
                            Vec2::from([PIPE_LENGTH, PIPE_WIDTH]),
                        ),
                        CornerRadius {
                            ne: 0,
                            nw: PIPE_WIDTH as u8 / 2,
                            se: 0,
                            sw: PIPE_WIDTH as u8 / 2,
                        },
                        color,
                    );
                }
            }
        }

        self.handle_interactions(&response, ui.ctx(), &canvas_rect);

        response
    }
}
impl FlowCanvas {
    pub fn with_size(width: usize, height: usize) -> Self {
        FlowCanvas {
            grid: flow_grid::FlowGrid::with_size(width, height),
            have_laid_pipe: false,
            previous_row_col: None,
            can_edit_sources: true,
        }
    }

    fn draw_grid_lines(&self, painter: &Painter, canvas_rect: &Rect, color: Color32) {
        for row in 0..=self.grid.height {
            let y = row as f32 * (CELL_SIZE + GRID_BORDER_WIDTH) + canvas_rect.min.y;
            painter.rect_filled(
                Rect::from_two_pos(
                    Pos2::new(canvas_rect.min.x, y),
                    Pos2::new(canvas_rect.max.x, y + GRID_BORDER_WIDTH),
                ),
                0,
                color,
            );
        }
        for col in 0..=self.grid.width {
            let x = col as f32 * (CELL_SIZE + GRID_BORDER_WIDTH) + canvas_rect.min.x;
            painter.rect_filled(
                Rect::from_two_pos(
                    Pos2::new(x, canvas_rect.min.y),
                    Pos2::new(x + GRID_BORDER_WIDTH, canvas_rect.max.y),
                ),
                0,
                color,
            );
        }
    }

    fn handle_interactions(&mut self, response: &Response, ctx: &Context, canvas_rect: &Rect) {
        let local_pos = if let Some(pointer_pos) = ctx.pointer_interact_pos() {
            pointer_pos - canvas_rect.min
        } else {
            return;
        };
        if local_pos.x < 0.0 || local_pos.y < 0.0 {
            return;
        }
        let row = (local_pos.y / CELL_SIZE).floor() as usize;
        let col = (local_pos.x / CELL_SIZE).floor() as usize;
        if row >= self.grid.height || col >= self.grid.width {
            return;
        }

        response.clicked().then(|| self.handle_clicked(row, col));
        response
            .drag_started()
            .then(|| self.handle_drag_start(row, col));
        response.dragged().then(|| self.handle_dragged(row, col));
        response
            .drag_stopped()
            .then(|| self.handle_drag_stopped(row, col));
    }

    fn handle_drag_start(&mut self, row: usize, col: usize) {
        if self.grid.get(row, col).unwrap().num_connections() > 1 {
            println!("TODO Started dragging in the middle of the pipe. Idk what I want to do.");
            // TODO if one end is connected to the source, disconnect the other end
            // if both ends connected or if neither end is connected, take the shortest path,
            // otherwise, just pick one, who cares.
            return;
        }
        self.previous_row_col = Some((row, col));
        self.have_laid_pipe = false;
    }

    fn handle_dragged(&mut self, row: usize, col: usize) {
        if let Some((prev_row, prev_col)) = self.previous_row_col {
            if prev_row == row && prev_col == col {
                return;
            }
            if let Some(direction) = Direction::try_from_adjacent(prev_row, prev_col, row, col) {
                let from_cell = self
                    .grid
                    .get(prev_row, prev_col)
                    .expect("we should only have stored cells that are valid");
                let to_cell = self
                    .grid
                    .get(row, col)
                    .expect("previously bounds checked indexes");

                if from_cell.is_direction_connected(direction) {
                    self.grid.try_disconnect(prev_row, prev_col, direction);
                } else if from_cell.color != to_cell.color {
                    // TODO add some logic that you can't switch colors mid-drag.
                    // For example, if you have . . .-.-. . . and then if you drag
                    // that entire width, you'd end up with .-.-. . .-.-.
                    self.grid.try_connect(prev_row, prev_col, direction);
                } else if self.grid.are_cells_connected(prev_row, prev_col, row, col) {
                    self.grid.remove_tail(row, col, prev_row, prev_col);
                } else {
                    self.grid.try_connect(prev_row, prev_col, direction);
                }
            } else {
                println!("TODO pathfinding");
                // TODO handle diagonals or fast mouse movements
            }
            self.have_laid_pipe = true;
        }
        self.previous_row_col = Some((row, col));
    }

    fn handle_drag_stopped(&mut self, row: usize, col: usize) {
        if !self.have_laid_pipe {
            self.handle_clicked(row, col)
        }
    }

    fn handle_clicked(&mut self, row: usize, col: usize) {
        if !self.can_edit_sources {
            return;
        }
        let cell = if let Some(cell) = self.grid.get(row, col) {
            cell
        } else {
            return;
        };

        if cell.is_source {
            self.grid.try_remove_source(row, col);
        } else {
            self.grid.try_set_new_source(row, col);
        }
    }
}

fn interpret_cell_color(color: CellColor) -> Color32 {
    match color {
        CellColor::Colored(color_id) => {
            if color_id < COLOR_INDEX.len() {
                COLOR_INDEX[color_id].1
            } else {
                Color32::BLACK
            }
        }
        CellColor::Empty(_) => Color32::from_rgb(0, 0, 0),
    }
}
