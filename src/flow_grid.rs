/// This file handles the core data model, abstracted away from any specific UI. you can ask for
/// various actions, and this will do validation and perform them.
use std::mem::swap;

pub struct FlowGrid {
    next_color_id: usize,
    cells: Vec<FlowCell>,
    pub width: usize,
    pub height: usize,
    source_index: Vec<(Option<usize>, Option<usize>)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn opposite(&self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    pub fn try_from_adjacent(
        row_from: usize,
        col_from: usize,
        row_to: usize,
        col_to: usize,
    ) -> Option<Self> {
        if row_from == row_to {
            if col_from + 1 == col_to {
                Some(Direction::Right)
            } else if col_from == col_to + 1 {
                Some(Direction::Left)
            } else {
                None
            }
        } else if col_from == col_to {
            if row_from + 1 == row_to {
                Some(Direction::Down)
            } else if row_from == row_to + 1 {
                Some(Direction::Up)
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellColor {
    Empty(usize),
    Colored(usize),
}

impl CellColor {
    pub fn can_colors_connect(color1: &CellColor, color2: &CellColor) -> bool {
        match (color1, color2) {
            (CellColor::Colored(id1), CellColor::Colored(id2)) => id1 == id2,
            _ => true,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FlowCell {
    pub color: CellColor,
    pub is_source: bool,
    pub is_connected_up: bool,
    pub is_connected_down: bool,
    pub is_connected_left: bool,
    pub is_connected_right: bool,
}

impl FlowCell {
    pub fn empty_with_id(empty_index: usize) -> Self {
        FlowCell {
            color: CellColor::Empty(empty_index),
            is_source: false,
            is_connected_up: false,
            is_connected_down: false,
            is_connected_left: false,
            is_connected_right: false,
        }
    }
    pub fn is_direction_connected(&self, direction: Direction) -> bool {
        match direction {
            Direction::Up => self.is_connected_up,
            Direction::Down => self.is_connected_down,
            Direction::Left => self.is_connected_left,
            Direction::Right => self.is_connected_right,
        }
    }

    fn add_connection(&mut self, direction: Direction) {
        match direction {
            Direction::Up => self.is_connected_up = true,
            Direction::Down => self.is_connected_down = true,
            Direction::Left => self.is_connected_left = true,
            Direction::Right => self.is_connected_right = true,
        }
    }

    fn remove_connection(&mut self, direction: Direction) {
        match direction {
            Direction::Up => self.is_connected_up = false,
            Direction::Down => self.is_connected_down = false,
            Direction::Left => self.is_connected_left = false,
            Direction::Right => self.is_connected_right = false,
        }
    }

    pub fn num_connections(&self) -> usize {
        let mut count = 0;
        if self.is_connected_up {
            count += 1;
        }
        if self.is_connected_down {
            count += 1;
        }
        if self.is_connected_left {
            count += 1;
        }
        if self.is_connected_right {
            count += 1;
        }
        count
    }

    pub fn has_open_connections(&self) -> bool {
        if self.num_connections() >= 2 {
            return false;
        }
        if self.is_source && self.num_connections() >= 1 {
            return false;
        }
        true
    }
}

impl FlowGrid {
    pub fn with_size(width: usize, height: usize) -> Self {
        let mut cells = Vec::with_capacity(width * height);
        for i in 0..(width * height) {
            cells.push(FlowCell::empty_with_id(i));
        }
        FlowGrid {
            next_color_id: 0,
            cells,
            width,
            height,
            source_index: Vec::new(),
        }
    }

    pub fn next_color(&self) -> usize {
        self.next_color_id
    }

    fn get_index(&self, row: usize, col: usize) -> Option<usize> {
        if row < self.height && col < self.width {
            Some(row * self.width + col)
        } else {
            None
        }
    }
    fn get_offset_index(&self, row: usize, col: usize, direction: Direction) -> Option<usize> {
        match direction {
            Direction::Up if row > 0 => self.get_index(row - 1, col),
            Direction::Down => self.get_index(row + 1, col),
            Direction::Left if col > 0 => self.get_index(row, col - 1),
            Direction::Right => self.get_index(row, col + 1),
            _ => None,
        }
    }
    fn offset_index(&self, index: usize, direction: Direction) -> Option<usize> {
        if index >= self.cells.len() {
            return None;
        }
        match direction {
            Direction::Up if index >= self.width => Some(index - self.width),
            Direction::Down if index + self.width < self.cells.len() => Some(index + self.width),
            Direction::Left if index % self.width > 0 => Some(index - 1),
            Direction::Right if index + 1 < self.cells.len() && (index + 1) % self.width != 0 => {
                Some(index + 1)
            }
            _ => None,
        }
    }
    pub fn get(&self, row: usize, col: usize) -> Option<&FlowCell> {
        self.cells.get(self.get_index(row, col)?)
    }
    pub fn offset_get(&self, row: usize, col: usize, direction: Direction) -> Option<&FlowCell> {
        self.cells.get(self.get_offset_index(row, col, direction)?)
    }
    fn get_mut(&mut self, row: usize, col: usize) -> Option<&mut FlowCell> {
        let index = self.get_index(row, col)?;
        self.cells.get_mut(index)
    }
    fn offset_get_mut(
        &mut self,
        row: usize,
        col: usize,
        direction: Direction,
    ) -> Option<&mut FlowCell> {
        let index = self.get_offset_index(row, col, direction)?;
        self.cells.get_mut(index)
    }

    pub fn get_offset_row_col(
        &self,
        row: usize,
        col: usize,
        direction: Direction,
    ) -> Option<(usize, usize)> {
        match direction {
            Direction::Up if row > 0 => Some((row - 1, col)),
            Direction::Down if row + 1 < self.height => Some((row + 1, col)),
            Direction::Left if col > 0 => Some((row, col - 1)),
            Direction::Right if col + 1 < self.width => Some((row, col + 1)),
            _ => None,
        }
    }

    pub fn try_set_new_source(&mut self, row: usize, col: usize) -> bool {
        if self.try_set_missing_source(row, col, self.next_color_id) {
            while let Some((Some(_), Some(_))) = self.source_index.get(self.next_color_id) {
                self.next_color_id += 1;
            }
            true
        } else {
            false
        }
    }

    pub fn try_set_missing_source(&mut self, row: usize, col: usize, color_id: usize) -> bool {
        let (index, cell) = if let Some(index) = self.get_index(row, col) {
            (index, self.cells[index])
        } else {
            println!("a");
            return false;
        };

        if cell.is_source {
            println!("b");
            return false;
        }

        if cell.num_connections() > 1 {
            println!("c");
            return false;
        }

        if !CellColor::can_colors_connect(&cell.color, &CellColor::Colored(color_id)) {
            println!("d");
            return false;
        }

        if let Some((prev_source1, prev_source2)) = self.source_index.get_mut(color_id) {
            if prev_source1.is_none() {
                *prev_source1 = Some(index);
            } else if prev_source2.is_none() {
                *prev_source2 = Some(index);
            } else {
                *prev_source1 = *prev_source2;
                *prev_source2 = Some(index);
            }
        } else {
            self.source_index
                .reserve(color_id - self.source_index.len() + 1);
            while self.source_index.len() < color_id {
                self.source_index.push((None, None));
            }
            self.source_index.push((Some(index), None));
        }

        let cell = self
            .get_mut(row, col)
            .expect("previously checked cells are in bounds");
        cell.is_source = true;
        cell.color = CellColor::Colored(color_id);

        if cell.is_connected_up {
            self.connect_core(
                self.offset_index(index, Direction::Up)
                    .expect("cells cannot be connects to the edge"),
                Direction::Down,
            );
        } else if cell.is_connected_down {
            self.connect_core(
                self.offset_index(index, Direction::Down)
                    .expect("cells cannot be connects to the edge"),
                Direction::Up,
            );
        } else if cell.is_connected_left {
            self.connect_core(
                self.offset_index(index, Direction::Left)
                    .expect("cells cannot be connects to the edge"),
                Direction::Right,
            );
        } else if cell.is_connected_right {
            self.connect_core(
                self.offset_index(index, Direction::Right)
                    .expect("cells cannot be connects to the edge"),
                Direction::Left,
            );
        }

        true
    }

    pub fn try_remove_source(&mut self, row: usize, col: usize) -> bool {
        let (index, cell) = if let Some(index) = self.get_index(row, col) {
            (index, &mut self.cells[index])
        } else {
            return false;
        };

        if !cell.is_source {
            return false;
        }

        let color_id = if let CellColor::Colored(color_id) = cell.color {
            color_id
        } else {
            panic!("sources should always have an explicit color");
        };

        cell.is_source = false;

        let index_entry = self
            .source_index
            .get_mut(color_id)
            .expect("All sources are registered in the index");
        if let Some(index1) = index_entry.0 {
            if index1 == index {
                index_entry.0 = index_entry.1;
                index_entry.1 = None;
            }
        }
        if let Some(index2) = index_entry.1 {
            if index2 == index {
                index_entry.1 = None;
            }
        }

        if color_id < self.next_color_id {
            self.next_color_id = color_id;
        }

        let should_decolor = cell.num_connections() == 0
            || match index_entry.0 {
                Some(other_index) => !self.are_cells_connected(
                    row,
                    col,
                    other_index / self.width,
                    other_index % self.width,
                ),
                None => true,
            };

        // cell needs to be dropped to call self.are_cells_connected, so this is
        // validating cell's value. I could probably use a Cell or something, but whatever
        let cell = &mut self.cells[index];
        if should_decolor {
            cell.color = CellColor::Empty(index);
            let direction = if cell.is_connected_down {
                Direction::Down
            } else if cell.is_connected_left {
                Direction::Left
            } else if cell.is_connected_right {
                Direction::Right
            } else if cell.is_connected_up {
                Direction::Up
            } else {
                return true;
            };
            self.connect_core(
                self.offset_index(index, direction)
                    .expect("cells should not be connected to the edge of the grid"),
                direction.opposite(),
            );
        }

        true
    }

    pub fn remove_tail(
        &mut self,
        base_row: usize,
        base_col: usize,
        tail_row: usize,
        tail_col: usize,
    ) -> bool {
        let mut tail_row = tail_row;
        let mut tail_col = tail_col;

        let tail = self.get(tail_row, tail_col);
        let mut tail = if let Some(tail) = tail {
            tail
        } else {
            return false;
        };

        if tail.num_connections() != 1 {
            return false;
        }
        let base = self.get(base_row, base_col);
        if let Some(base) = base {
            if base.color != tail.color {
                return false;
            }
        } else {
            return false;
        }

        while tail_row != base_row || tail_col != base_col {
            let direction = if tail.is_connected_down {
                Direction::Down
            } else if tail.is_connected_up {
                Direction::Up
            } else if tail.is_connected_left {
                Direction::Left
            } else if tail.is_connected_right {
                Direction::Right
            } else {
                return false;
            };
            if !self.try_disconnect(tail_row, tail_col, direction) {
                return false;
            }

            (tail_row, tail_col) = self
                .get_offset_row_col(tail_row, tail_col, direction)
                .expect("Grid should not connect to the edges");
            tail = self
                .get(tail_row, tail_col)
                .expect("previously checked cells are in bounds");
        }

        true
    }

    pub fn try_disconnect(&mut self, row: usize, col: usize, direction: Direction) -> bool {
        let index = self.get_index(row, col);
        let other_index = self.get_offset_index(row, col, direction);
        let (index, other_index) = match (index, other_index) {
            (Some(i), Some(oi)) => (i, oi),
            _ => return false,
        };

        let cell = self.cells[index];
        let offset_cell = self.cells[other_index];

        if !cell.is_direction_connected(direction) {
            return false;
        }
        if !offset_cell.is_direction_connected(direction.opposite()) {
            return false;
        }

        let cell = self
            .get_mut(row, col)
            .expect("previously checked cells are in bounds");
        cell.remove_connection(direction);
        if cell.num_connections() == 0 && !cell.is_source {
            cell.color = CellColor::Empty(index);
        }

        let offset_cell = self
            .offset_get_mut(row, col, direction)
            .expect("previously checked cells are in bounds");
        offset_cell.remove_connection(direction.opposite());
        if offset_cell.num_connections() == 0 && !offset_cell.is_source {
            offset_cell.color = CellColor::Empty(other_index);
        }

        true
    }

    pub fn try_connect(&mut self, row: usize, col: usize, direction: Direction) -> bool {
        let cell1 = self.get(row, col);
        let cell2 = self.offset_get(row, col, direction);

        if cell1.is_none() || cell2.is_none() {
            return false;
        }
        let cell1 = cell1.unwrap();
        let cell2 = cell2.unwrap();

        if !cell1.has_open_connections() || !cell2.has_open_connections() {
            return false;
        }

        if cell1.is_direction_connected(direction)
            || cell2.is_direction_connected(direction.opposite())
        {
            return false;
        }

        if !CellColor::can_colors_connect(&cell1.color, &cell2.color) {
            return false;
        }

        let mut core_params1 = (
            self.get_index(row, col)
                .expect("previous validation verifies this is a valid index"),
            direction,
        );
        let mut core_params2 = (
            self.get_offset_index(row, col, direction)
                .expect("previous validation verifies this is a valid index"),
            direction.opposite(),
        );

        if let CellColor::Colored(_) = cell1.color {
            swap(&mut core_params1, &mut core_params2);
        }

        self.connect_core(core_params1.0, core_params1.1);
        self.connect_core(core_params2.0, core_params2.1);

        true
    }

    fn connect_core(&mut self, index: usize, direction: Direction) {
        let mut index = index;
        let mut direction = direction;

        loop {
            let new_color = self.cells[self.offset_index(index, direction).unwrap()].color;
            let cell = &mut self.cells[index];

            cell.add_connection(direction);

            if new_color == cell.color {
                break;
            }
            cell.color = new_color;
            let cell = &self.cells[index];

            let next_params: Option<(usize, Direction)> = if cell.is_connected_up
                && new_color != self.cells[self.offset_index(index, Direction::Up).unwrap()].color
            {
                self.offset_index(index, Direction::Up)
                    .map(|next_index| (next_index, Direction::Down))
            } else if cell.is_connected_down
                && new_color != self.cells[self.offset_index(index, Direction::Down).unwrap()].color
            {
                self.offset_index(index, Direction::Down)
                    .map(|next_index| (next_index, Direction::Up))
            } else if cell.is_connected_left
                && new_color != self.cells[self.offset_index(index, Direction::Left).unwrap()].color
            {
                self.offset_index(index, Direction::Left)
                    .map(|next_index| (next_index, Direction::Right))
            } else if cell.is_connected_right
                && new_color
                    != self.cells[self.offset_index(index, Direction::Right).unwrap()].color
            {
                self.offset_index(index, Direction::Right)
                    .map(|next_index| (next_index, Direction::Left))
            } else {
                None
            };
            if let Some((next_index, next_direction)) = next_params {
                index = next_index;
                direction = next_direction;
            } else {
                break;
            }
        }
    }

    pub fn are_cells_connected(&self, row1: usize, col1: usize, row2: usize, col2: usize) -> bool {
        if row1 == row2 && col1 == col2 {
            return true;
        }
        let (index1, cell1) = if let Some(index) = self.get_index(row1, col1) {
            (index, self.cells[index])
        } else {
            return false;
        };
        let (index2, cell2) = if let Some(index) = self.get_index(row2, col2) {
            (index, self.cells[index])
        } else {
            return false;
        };
        if cell1.color != cell2.color {
            return false;
        }
        self.are_cells_connected_core(None, index1, None, index2)
    }

    fn are_cells_connected_core(
        &self,
        original_index: Option<usize>,
        from_index: usize,
        from_direction: Option<Direction>,
        to_index: usize,
    ) -> bool {
        if Some(from_index) == original_index {
            return false;
        }
        if from_index == to_index {
            return true;
        }

        let cell = &self.cells[from_index];
        if cell.is_connected_up && from_direction != Some(Direction::Up) {
            if let Some(next_index) = self.offset_index(from_index, Direction::Up) {
                if self.are_cells_connected_core(
                    original_index.or(Some(from_index)),
                    next_index,
                    Some(Direction::Down),
                    to_index,
                ) {
                    return true;
                }
            }
        }

        if cell.is_connected_down && from_direction != Some(Direction::Down) {
            if let Some(next_index) = self.offset_index(from_index, Direction::Down) {
                if self.are_cells_connected_core(
                    original_index.or(Some(from_index)),
                    next_index,
                    Some(Direction::Up),
                    to_index,
                ) {
                    return true;
                }
            }
        }

        if cell.is_connected_left && from_direction != Some(Direction::Left) {
            if let Some(next_index) = self.offset_index(from_index, Direction::Left) {
                if self.are_cells_connected_core(
                    original_index.or(Some(from_index)),
                    next_index,
                    Some(Direction::Right),
                    to_index,
                ) {
                    return true;
                }
            }
        }

        if cell.is_connected_right && from_direction != Some(Direction::Right) {
            if let Some(next_index) = self.offset_index(from_index, Direction::Right) {
                if self.are_cells_connected_core(
                    original_index.or(Some(from_index)),
                    next_index,
                    Some(Direction::Left),
                    to_index,
                ) {
                    return true;
                }
            }
        }

        false
    }
}
