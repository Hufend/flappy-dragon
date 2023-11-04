use bracket_lib::prelude::*;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const FRAME_DURATION: f32 = 50.0;
// 模式枚举
enum Mode {
    Menu,
    Play,
    Pausing,
    Ending,
}
// 玩家结构
struct Player {
    // x + location 为真实坐标
    x: i32,
    y: i32,
    location: i32, // 渲染偏移量
    velocity: f32, // 掉落加速度
}
// 障碍结构
struct Obstacle {
    x: i32,
    mid_gap: i32, // 空隙中间点坐标
    size: i32,    // 空隙大小
}
// 状态结构
struct State {
    player: Player,
    obstacles: Vec<Obstacle>,
    score: i32,
    time: f32,
    mode: Mode,
}
// 玩家结构的处理函数
impl Player {
    // 结构构建
    fn new(location: i32) -> Self {
        Player {
            x: 0,
            y: 0,
            location,
            velocity: 0.0,
        }
    }
    // 图形渲染
    fn render(&mut self, ctx: &mut BTerm) {
        ctx.set(self.location, self.y, YELLOW, BLACK, to_cp437('@'));
    }
    // 位置更新
    fn update(&mut self) {
        if self.velocity < 2.0 {
            self.velocity += 0.2;
        }
        self.y += self.velocity.round() as i32;
        self.x += 1;
        if self.y < 0 {
            self.y = 0;
        }
    }
    // 点击
    fn flap(&mut self) {
        self.velocity = -2.0;
    }
    // 移动
    fn movement(&mut self, value: i32) {
        self.location += value;
        if self.location < 0 {
            self.location = 0;
        } else if self.location > SCREEN_WIDTH - 1 {
            self.location = SCREEN_WIDTH - 1;
        }
    }
}
// 障碍结构的处理函数
impl Obstacle {
    // 结构构建
    fn new(x: i32, score: i32) -> Self {
        let mut random = RandomNumberGenerator::new();
        Obstacle {
            x,
            mid_gap: random.range(10, 40),
            size: i32::max(2, 25 - score / 4),
        }
    }
    // 图形渲染
    fn render(&mut self, ctx: &mut BTerm, player_x: i32) {
        let x = self.x - player_x; // 屏幕上的相对坐标
        let half_size = self.size / 2;
        for y in 0..self.mid_gap - half_size {
            ctx.set(x, y, RED, BLACK, to_cp437('■'));
        }
        for y in self.mid_gap + half_size..SCREEN_HEIGHT {
            ctx.set(x, y, RED, BLACK, to_cp437('■'));
        }
    }
    // 碰撞判断
    fn judge(&self, player: &Player) -> bool {
        let half_size = self.size / 2;
        player.x + player.location == self.x
            && ((player.y < self.mid_gap - half_size) || (player.y > self.mid_gap + half_size))
    }
}
// 状态结构的处理函数
impl State {
    // 结构构建
    fn new() -> Self {
        State {
            player: Player::new(0),
            obstacles: Vec::new(),
            score: 0,
            time: 0.0,
            mode: Mode::Menu,
        }
    }
    // 菜单
    fn menu(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "Welcome to Flappy Dragon @!");
        ctx.print_centered(8, "(P) Play");
        ctx.print_centered(9, "(Q) Quit");
        self.menu_key_callback(ctx);
    }
    // 游戏界面
    fn play(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(NAVY);
        self.time += ctx.frame_time_ms;
        if self.time > FRAME_DURATION {
            self.time = 0.0;
            self.player.update();
        }
        self.player.render(ctx);
        // 遍历 obstacles 渲染所有障碍物
        for i in &mut self.obstacles {
            i.render(ctx, self.player.x);
        }

        ctx.print_centered(0, &format!("Score: {}", self.score));
        ctx.print(0, 0, "Press Space to flap");
        ctx.print(0, 1, "Press A, D to move");
        // 当障碍物超出屏幕时得分
        if self.player.x > self.obstacles[0].x {
            self.score += 1;
            self.obstacles.remove(0);
            self.obstacles
                .push(Obstacle::new(self.player.x + SCREEN_WIDTH, self.score + 3));
        }
        // 失败条件
        // 玩家坠落
        let mut is_end = self.player.y > SCREEN_HEIGHT;
        // 碰到任意一个障碍物
        for i in &mut self.obstacles {
            is_end = is_end || i.judge(&self.player);
        }
        // 游戏结束
        if is_end {
            self.mode = Mode::Ending;
        }

        self.play_key_callback(ctx);
    }
    // 暂停界面
    fn pausing(&mut self, ctx: &mut BTerm) {
        ctx.print_centered(5, "Game Pause!");
        ctx.print_centered(8, "(C) Continue");
        ctx.print_centered(9, "(R) Restart");
        ctx.print_centered(10, "(Q) Back to main menu");
        self.pausing_key_callback(ctx);
    }
    // 结束界面
    fn ending(&mut self, ctx: &mut BTerm) {
        ctx.print_centered(5, "Game Over!");
        ctx.print_centered(8, "(P) Play again");
        ctx.print_centered(9, "(Q) Back to main menu");
        self.ending_key_callback(ctx);
    }
    // 菜单按键绑定
    fn menu_key_callback(&mut self, ctx: &mut BTerm) {
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.start(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }
    // 游戏按键绑定
    fn play_key_callback(&mut self, ctx: &mut BTerm) {
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::Space => self.player.flap(),
                VirtualKeyCode::D => self.player.movement(1),
                VirtualKeyCode::A => self.player.movement(-1),
                VirtualKeyCode::P => self.pause(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }
    // 暂停按键绑定
    fn pausing_key_callback(&mut self, ctx: &mut BTerm) {
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::C => self.continue_(),
                VirtualKeyCode::R => self.start(),
                VirtualKeyCode::Q => self.mode = Mode::Menu,
                _ => {}
            }
        }
    } // 结束按键绑定
    fn ending_key_callback(&mut self, ctx: &mut BTerm) {
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.start(),
                VirtualKeyCode::Q => self.mode = Mode::Menu,
                _ => {}
            }
        }
    }
    // 游戏开始
    fn start(&mut self) {
        self.player = Player::new(0);
        self.score = 0;
        self.time = 0.0;
        self.mode = Mode::Play;
        self.obstacles.clear();
        self.obstacles.push(Obstacle::new(SCREEN_WIDTH, self.score));
        self.obstacles
            .push(Obstacle::new(SCREEN_WIDTH / 4 * 5, self.score + 1));
        self.obstacles
            .push(Obstacle::new(SCREEN_WIDTH / 2 * 3, self.score + 2));
        self.obstacles
            .push(Obstacle::new(SCREEN_WIDTH / 4 * 7, self.score + 3));
    }
    // 游戏暂停
    fn pause(&mut self) {
        self.mode = Mode::Pausing;
    }
    // 游戏继续
    fn continue_(&mut self) {
        self.mode = Mode::Play;
    }
}
// 实现函数 tick
impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        match self.mode {
            Mode::Menu => self.menu(ctx),
            Mode::Play => self.play(ctx),
            Mode::Pausing => self.pausing(ctx),
            Mode::Ending => self.ending(ctx),
        }
    }
}
// 程序入口
fn main() -> BError {
    let context = BTermBuilder::simple80x50()
        .with_title("Flappy Dragon")
        .build()?;
    main_loop(context, State::new())
}
