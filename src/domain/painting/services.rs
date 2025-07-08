use crate::domain::artwork::entities::{Artwork, Canvas};
use crate::domain::controller::{
    Button, ControllerAction, ControllerCommand, DPad,
};
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
        info!("Generated drawing path with {} dots", drawing_path.coordinates.len());

        // 4. 描画コマンドを生成
        let drawing_commands = self.create_drawing_commands(&drawing_path);
        commands.extend(drawing_commands);

        // 5. 完了コマンド
        commands.push(self.create_completion_command());

        commands
    }

    /// 初期化コマンドを作成
    fn create_initialization_command(&self) -> ControllerCommand {
        let mut command = ControllerCommand::new("Initialize")
            .with_description("コントローラーを初期化");

        // Switchのメニュー表示待機
        command = command
            .add_action(ControllerAction::wait(2000))
            .add_action(ControllerAction::set_dpad(DPad::NEUTRAL, 100));
        
        // カーソルを左上に移動（初期位置へ）
        // 左上に完全に移動するため、画面サイズ以上の移動を実行
        for _ in 0..150 {
            command = command.add_action(ControllerAction::set_dpad(DPad::UP_LEFT, 20));
        }
        command = command
            .add_action(ControllerAction::set_dpad(DPad::NEUTRAL, 500));

        command
    }

    /// 描画モード選択コマンドを作成
    fn create_select_drawing_mode_command(&self) -> ControllerCommand {
        let mut command = ControllerCommand::new("Select Drawing Mode")
            .with_description("描画モードを選択");

        let mode_button = self.config.drawing_mode.select_button();
        
        // ペン選択前の待機時間を追加
        command = command
            .add_action(ControllerAction::wait(500))
            .add_action(ControllerAction::press_button(mode_button, 200))
            .add_action(ControllerAction::release_button(mode_button, 1000));

        command
    }

    /// 描画パスを生成
    fn create_drawing_path(&self, canvas: &Canvas) -> DrawingPath {
        let drawable_dots = canvas.drawable_dots();
        let coordinates: Vec<Coordinates> = match self.strategy {
            DrawingStrategy::RasterScan => {
                // 左から右、上から下
                let mut coords: Vec<Coordinates> = drawable_dots.into_iter()
                    .map(|(coord, _)| *coord)
                    .collect();
                coords.sort_by_key(|c| (c.y, c.x));
                coords
            }
            DrawingStrategy::ZigZag => {
                // ジグザグパターン
                let mut coords: Vec<Coordinates> = drawable_dots.into_iter()
                    .map(|(coord, _)| *coord)
                    .collect();
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
            DrawingStrategy::Spiral => {
                // スパイラルパターン（未実装、ラスタースキャンにフォールバック）
                let mut coords: Vec<Coordinates> = drawable_dots.into_iter()
                    .map(|(coord, _)| *coord)
                    .collect();
                coords.sort_by_key(|c| (c.y, c.x));
                coords
            }
        };

        let mut path = DrawingPath::new(coordinates);
        path.calculate_estimated_time(&self.config);
        path
    }

    /// 最近傍探索でパスを生成
    fn nearest_neighbor_path(&self, drawable_dots: Vec<(&Coordinates, &crate::domain::artwork::entities::Dot)>) -> Vec<Coordinates> {
        if drawable_dots.is_empty() {
            return Vec::new();
        }

        let mut remaining: Vec<_> = drawable_dots.into_iter()
            .map(|(coord, _)| *coord)
            .collect();
        let mut path = Vec::new();
        
        // 最初の点（左上）
        let start_idx = remaining.iter()
            .position(|c| c.x == remaining.iter().map(|c| c.x).min().unwrap_or(0))
            .unwrap_or(0);
        let mut current = remaining.remove(start_idx);
        path.push(current);

        // 最近傍を探しながらパスを構築
        while !remaining.is_empty() {
            let nearest_idx = remaining.iter()
                .enumerate()
                .min_by_key(|(_, coord)| current.manhattan_distance_to(coord))
                .map(|(idx, _)| idx)
                .unwrap_or(0);
            
            current = remaining.remove(nearest_idx);
            path.push(current);
        }

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
                    .add_action(ControllerAction::press_button(Button::A, self.config.dot_draw_delay_ms))
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