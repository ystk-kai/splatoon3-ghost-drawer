use crate::domain::artwork::entities::{Artwork, Canvas};
use crate::domain::controller::{Button, ControllerAction, ControllerCommand, DPad};
use crate::domain::painting::value_objects::{
    CursorDirection, DrawingCanvasConfig, DrawingPath, DrawingStrategy,
};
use crate::domain::shared::value_objects::Coordinates;
use tracing::info;

/// アートワークをコントローラーコマンドに変換するサービス
pub struct ArtworkToCommandConverter {
    config: DrawingCanvasConfig,
    strategy: DrawingStrategy,
}

impl ArtworkToCommandConverter {
    pub fn new(config: DrawingCanvasConfig, strategy: DrawingStrategy) -> Self {
        Self { config, strategy }
    }

    /// アートワークをコントローラーコマンドのシーケンスに変換
    pub fn convert(&self, artwork: &Artwork) -> Vec<ControllerCommand> {
        let mut commands = Vec::new();

        // 1. 初期化コマンド
        commands.push(self.create_initialization_command());

        // 2. 描画モード選択コマンド
        commands.push(self.create_select_drawing_mode_command());

        // 3. 描画パスを生成
        let drawing_path = self.create_drawing_path(&artwork.canvas);
        info!(
            "Generated drawing path with {} dots",
            drawing_path.coordinates.len()
        );

        // 4. 描画コマンドを生成
        let drawing_commands = self.create_drawing_commands(&drawing_path);
        commands.extend(drawing_commands);

        // 5. 完了コマンド
        commands.push(self.create_completion_command());

        commands
    }

    /// 初期化コマンドを作成
    fn create_initialization_command(&self) -> ControllerCommand {
        let mut command =
            ControllerCommand::new("Initialize").with_description("コントローラーを初期化");

        // Switchのメニュー表示待機
        command = command
            .add_action(ControllerAction::wait(2000))
            .add_action(ControllerAction::set_dpad(DPad::NEUTRAL, 100));

        // カーソルを左上に移動（初期位置へ）
        // 左上に完全に移動するため、画面サイズ以上の移動を実行
        // よりゆっくりとした動作で移動
        for _ in 0..150 {
            command = command
                .add_action(ControllerAction::set_dpad(DPad::UP_LEFT, 50))
                .add_action(ControllerAction::set_dpad(DPad::NEUTRAL, 50));
        }
        command = command.add_action(ControllerAction::set_dpad(DPad::NEUTRAL, 500));

        command
    }

    /// 描画モード選択コマンドを作成
    fn create_select_drawing_mode_command(&self) -> ControllerCommand {
        let mut command =
            ControllerCommand::new("Select Drawing Mode").with_description("描画モードを選択");

        let mode_button = self.config.drawing_mode.select_button();

        // ペン選択前の待機時間を追加
        command = command.add_action(ControllerAction::wait(500));

        // Lボタンの場合は最低2回押す（ピクセルペンを確実に選択）
        if mode_button == Button::L {
            // ピクセルペンに切り替えるために2回押す
            for i in 0..2 {
                command = command
                    .add_action(ControllerAction::press_button(Button::L, 200))
                    .add_action(ControllerAction::release_button(Button::L, 300));
                if i < 1 {
                    command = command.add_action(ControllerAction::wait(500));
                }
            }
        } else {
            command = command
                .add_action(ControllerAction::press_button(mode_button, 200))
                .add_action(ControllerAction::release_button(mode_button, 300));
        }

        command = command.add_action(ControllerAction::wait(1000));

        command
    }

    /// 描画パスを生成
    pub fn create_drawing_path(&self, canvas: &Canvas) -> DrawingPath {
        let drawable_dots = canvas.drawable_dots();
        let coordinates: Vec<Coordinates> = match self.strategy {
            DrawingStrategy::RasterScan => {
                // 左から右、上から下
                let mut coords: Vec<Coordinates> =
                    drawable_dots.into_iter().map(|(coord, _)| *coord).collect();
                coords.sort_by_key(|c| (c.y, c.x));
                coords
            }
            DrawingStrategy::ZigZag => {
                // ジグザグパターン
                let mut coords: Vec<Coordinates> =
                    drawable_dots.into_iter().map(|(coord, _)| *coord).collect();
                coords.sort_by_key(|c| (c.y, c.x));

                // 偶数行は逆順にする
                let mut result = Vec::new();
                let mut current_y = 0;
                let mut row = Vec::new();

                for coord in coords {
                    if coord.y != current_y {
                        if current_y % 2 == 1 {
                            row.reverse();
                        }
                        result.extend(row);
                        row = Vec::new();
                        current_y = coord.y;
                    }
                    row.push(coord);
                }

                if current_y % 2 == 1 {
                    row.reverse();
                }
                result.extend(row);
                result
            }
            DrawingStrategy::NearestNeighbor => {
                // 最近傍探索（簡易版）
                self.nearest_neighbor_path(drawable_dots)
            }
            DrawingStrategy::GreedyTwoOpt => {
                // Greedy + 2-opt最適化
                let path = self.nearest_neighbor_path(drawable_dots);
                self.two_opt_optimize(path)
            }
            DrawingStrategy::Spiral => {
                // スパイラルパターン（未実装、ラスタースキャンにフォールバック）
                let mut coords: Vec<Coordinates> =
                    drawable_dots.into_iter().map(|(coord, _)| *coord).collect();
                coords.sort_by_key(|c| (c.y, c.x));
                coords
            }
        };

        let mut path = DrawingPath::new(coordinates);
        path.calculate_estimated_time(&self.config);
        path
    }

    /// 最近傍探索でパスを生成（グリッド最適化版）
    fn nearest_neighbor_path(
        &self,
        drawable_dots: Vec<(&Coordinates, &crate::domain::artwork::entities::Dot)>,
    ) -> Vec<Coordinates> {
        if drawable_dots.is_empty() {
            return Vec::new();
        }

        let total_dots = drawable_dots.len();
        let mut path = Vec::with_capacity(total_dots);

        // グリッドサイズ（バケットサイズ）
        // 320x120のキャンバスに対して10x10のグリッドを作成
        const GRID_SIZE: i16 = 10;
        const GRID_COLS: usize = (320 / GRID_SIZE as usize) + 1;
        const GRID_ROWS: usize = (120 / GRID_SIZE as usize) + 1;

        // グリッドの初期化
        let mut grid: Vec<Vec<Vec<Coordinates>>> = vec![vec![Vec::new(); GRID_COLS]; GRID_ROWS];
        
        // 全点をグリッドに配置
        for (coord, _) in drawable_dots {
            let col = (coord.x as usize) / (GRID_SIZE as usize);
            let row = (coord.y as usize) / (GRID_SIZE as usize);
            if row < GRID_ROWS && col < GRID_COLS {
                grid[row][col].push(*coord);
            }
        }

        // 最初の点（左上）を探す
        // グリッドの左上から順に探して最初に見つかった点を使用
        let mut current = Coordinates::new(0, 0);
        let mut found_start = false;
        
        'start_search: for row in 0..GRID_ROWS {
            for col in 0..GRID_COLS {
                if !grid[row][col].is_empty() {
                    // バケット内で最も左上の点を探す
                    let mut min_idx = 0;
                    let mut min_val = i32::MAX;
                    
                    for (i, p) in grid[row][col].iter().enumerate() {
                        let val = p.x as i32 + p.y as i32;
                        if val < min_val {
                            min_val = val;
                            min_idx = i;
                        }
                    }
                    
                    current = grid[row][col].swap_remove(min_idx);
                    path.push(current);
                    found_start = true;
                    break 'start_search;
                }
            }
        }

        if !found_start {
            return Vec::new();
        }

        // 残りの点を探索
        for _ in 1..total_dots {
            let current_col = (current.x as usize) / (GRID_SIZE as usize);
            let current_row = (current.y as usize) / (GRID_SIZE as usize);
            
            let mut nearest_dist = u32::MAX;
            let mut nearest_point = Coordinates::new(0, 0);
            let mut found_bucket_row = 0;
            let mut found_bucket_col = 0;
            let mut found_idx = 0;
            let mut found = false;

            // 近隣のバケットから探索範囲を広げていく
            // 半径0（自身のバケット）から開始
            let max_radius = std::cmp::max(GRID_ROWS, GRID_COLS);
            
            'search: for radius in 0..=max_radius {
                // 探索範囲のバケットをチェック
                let r_min = (current_row as isize - radius as isize).max(0) as usize;
                let r_max = (current_row as isize + radius as isize).min(GRID_ROWS as isize - 1) as usize;
                let c_min = (current_col as isize - radius as isize).max(0) as usize;
                let c_max = (current_col as isize + radius as isize).min(GRID_COLS as isize - 1) as usize;

                let mut found_in_radius = false;

                for r in r_min..=r_max {
                    for c in c_min..=c_max {
                        // 半径のエッジにあるバケットのみをチェック（内側は既にチェック済み）
                        // ただしradius=0の場合はチェックする
                        let is_edge = radius == 0 || 
                                      r == r_min || r == r_max || 
                                      c == c_min || c == c_max;
                        
                        if is_edge && !grid[r][c].is_empty() {
                            for (i, p) in grid[r][c].iter().enumerate() {
                                let dist = current.manhattan_distance_to(p);
                                if dist < nearest_dist {
                                    nearest_dist = dist;
                                    nearest_point = *p;
                                    found_bucket_row = r;
                                    found_bucket_col = c;
                                    found_idx = i;
                                    found = true;
                                    found_in_radius = true;
                                }
                            }
                        }
                    }
                }

                // この半径で見つかり、かつ次の半径の最小距離よりも近ければ確定
                // （マンハッタン距離なので、グリッド境界までの距離を考慮する必要があるが、
                //  簡易的に「見つかったら終了」とする。厳密な最近傍でなくても十分）
                if found_in_radius {
                    break 'search;
                }
            }

            if found {
                // 見つかった点を削除してパスに追加
                grid[found_bucket_row][found_bucket_col].swap_remove(found_idx);
                current = nearest_point;
                path.push(current);
            } else {
                break; // 点が見つからない（通常ありえない）
            }
        }

        path
    }

    /// 2-optアルゴリズムによるパスの最適化
    fn two_opt_optimize(&self, mut path: Vec<Coordinates>) -> Vec<Coordinates> {
        let n = path.len();
        if n < 4 {
            return path;
        }

        let mut improved = true;
        let mut iterations = 0;
        // 無限ループ防止と処理時間制限のための最大反復回数
        const MAX_ITERATIONS: usize = 50;

        // 探索ウィンドウサイズ（近傍のみを探索して計算量を削減）
        // 全点対全点だとO(N^2)で38400点の場合に数分かかるため、
        // 前後500点程度に制限してO(N*K)にする
        const WINDOW_SIZE: usize = 500;

        while improved && iterations < MAX_ITERATIONS {
            improved = false;
            iterations += 1;

            for i in 0..n - 2 {
                // jはi+2から開始し、ウィンドウサイズまたは配列末尾まで
                let end_j = std::cmp::min(i + WINDOW_SIZE, n - 1);
                
                for j in i + 2..end_j {
                    let p1 = path[i];
                    let p2 = path[i + 1];
                    let p3 = path[j];
                    let p4 = path[j + 1];

                    // 現在の距離（p1->p2 + p3->p4）
                    let current_dist = p1.manhattan_distance_to(&p2) + p3.manhattan_distance_to(&p4);
                    // 交換後の距離（p1->p3 + p2->p4）
                    // p1からp3へ行き、そこから逆順にp2へ戻り、p4へ向かう
                    let new_dist = p1.manhattan_distance_to(&p3) + p2.manhattan_distance_to(&p4);

                    if new_dist < current_dist {
                        // セグメント[i+1..=j]を反転
                        path[i + 1..=j].reverse();
                        improved = true;
                    }
                }
            }
        }

        info!(
            "2-opt optimization finished after {} iterations",
            iterations
        );

        path
    }

    /// 描画コマンドを生成
    fn create_drawing_commands(&self, path: &DrawingPath) -> Vec<ControllerCommand> {
        let mut commands = Vec::new();
        let mut current_pos = Coordinates::new(0, 0); // 開始位置

        // バッチサイズ（1コマンドあたりのドット数）
        const BATCH_SIZE: usize = 100;

        for (batch_idx, chunk) in path.coordinates.chunks(BATCH_SIZE).enumerate() {
            let mut command = ControllerCommand::new(format!("Draw Batch {}", batch_idx + 1))
                .with_description(format!("{}個のドットを描画", chunk.len()));

            for target in chunk {
                // 現在位置から目標位置への移動コマンドを追加
                let move_actions = self.create_move_actions(&current_pos, target);
                for action in move_actions {
                    command = command.add_action(action);
                }

                // ドットを描画
                command = command
                    .add_action(ControllerAction::press_button(
                        Button::A,
                        self.config.dot_draw_delay_ms,
                    ))
                    .add_action(ControllerAction::release_button(Button::A, 50));

                current_pos = *target;
            }

            commands.push(command);
        }

        commands
    }

    /// 2点間の移動アクションを生成
    fn create_move_actions(&self, from: &Coordinates, to: &Coordinates) -> Vec<ControllerAction> {
        let mut actions = Vec::new();
        let mut current = *from;

        // X方向の移動
        while current.x != to.x {
            let direction = if to.x > current.x {
                CursorDirection::Right
            } else {
                CursorDirection::Left
            };

            actions.push(ControllerAction::set_dpad(
                direction.to_dpad(),
                self.config.cursor_speed_ms,
            ));

            if to.x > current.x {
                current.x += 1;
            } else {
                current.x -= 1;
            }
        }

        // Y方向の移動
        while current.y != to.y {
            let direction = if to.y > current.y {
                CursorDirection::Down
            } else {
                CursorDirection::Up
            };

            actions.push(ControllerAction::set_dpad(
                direction.to_dpad(),
                self.config.cursor_speed_ms,
            ));

            if to.y > current.y {
                current.y += 1;
            } else {
                current.y -= 1;
            }
        }

        // 移動後はニュートラルに戻す
        if !actions.is_empty() {
            actions.push(ControllerAction::set_dpad(DPad::NEUTRAL, 50));
        }

        actions
    }

    /// 完了コマンドを作成
    fn create_completion_command(&self) -> ControllerCommand {
        ControllerCommand::new("Complete")
            .with_description("描画完了")
            .add_action(ControllerAction::wait(1000))
            .add_action(ControllerAction::press_button(Button::HOME, 100))
            .add_action(ControllerAction::release_button(Button::HOME, 500))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::painting::value_objects::DrawingMode;

    #[test]
    fn test_two_opt_optimize_removes_crossing() {
        let config = DrawingCanvasConfig {
            width: 100,
            height: 100,
            cursor_speed_ms: 10,
            dot_draw_delay_ms: 10,
            line_wrap_delay_ms: 10,
            drawing_mode: DrawingMode::PixelPen,
        };
        let strategy = DrawingStrategy::GreedyTwoOpt;
        let converter = ArtworkToCommandConverter::new(config, strategy);

        // Create a path with a crossing: (0,0) -> (10,10) -> (0,10) -> (10,0)
        // Crosses like an X
        let path = vec![
            Coordinates::new(0, 0),
            Coordinates::new(10, 10),
            Coordinates::new(0, 10),
            Coordinates::new(10, 0),
        ];

        let optimized = converter.two_opt_optimize(path.clone());

        // Calculate distances
        let original_dist: u32 = path
            .windows(2)
            .map(|w| w[0].manhattan_distance_to(&w[1]))
            .sum();
            
        let optimized_dist: u32 = optimized
            .windows(2)
            .map(|w| w[0].manhattan_distance_to(&w[1]))
            .sum();

        println!("Original distance: {}", original_dist);
        println!("Optimized distance: {}", optimized_dist);

        assert!(optimized_dist < original_dist, "Optimized path should be shorter");
        assert_eq!(optimized.len(), path.len(), "Path length should be preserved");
        
        // Check if start point is preserved (optional, but usually desired for first point)
        // Note: 2-opt might reverse the whole path or segments, but usually we fix start if needed.
        // In my implementation, I start swapping from i=0, so path[0] is fixed as p1 in the first iteration?
        // Actually, i goes from 0 to n-2. p1 = path[i]. 
        // If i=0, p1=path[0]. We swap path[i+1..=j]. So path[0] is never moved.
        assert_eq!(optimized[0], path[0], "Start point should be preserved");
    }
}
