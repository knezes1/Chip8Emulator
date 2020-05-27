mod chip_8_emulator;
use ggez;
use ggez::event;
use ggez::graphics;
use ggez::{Context, GameResult};
use ggez::nalgebra;
use ggez::event::{KeyCode, KeyMods};

struct MainState {
    chip_8: chip_8_emulator::Chip8Hardware,
    start_x: f32,
    start_y: f32,
    opcode_value: u16,
}

impl MainState {
    fn new() -> GameResult<MainState> {
        let mut c_8 = chip_8_emulator::Chip8Hardware::new();
        c_8.cpu_reset();
        c_8.load_game("games/Chip8Picture.ch8".to_string());
        let s = MainState {
            chip_8: c_8,
            start_x: 0.0,
            start_y: 50.0,
            opcode_value: 0,
        };
        Ok(s)
    }
}

impl event::EventHandler for MainState{

    fn update(&mut self, _ctx: &mut Context) -> GameResult{

        self.opcode_value = 0;
        self.chip_8.fetch_opcode(&mut self.opcode_value);
        let opc = self.opcode_value;
        self.chip_8.decode_and_execute_opcode(opc);
        self.chip_8.decrement_timer_counter();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {

        let mut screen = graphics::MeshBuilder::new();

        if self.chip_8.get_draw_enabled() == true{
            for y in 0..32{
                for x in 0..64{
                    let pixel_color = if self.chip_8.get_pixel_value_x_y(y, x) == true {graphics::WHITE} else {graphics::BLACK};
                    let x_coord = x as f32 * 12.5 + self.start_x;
                    let y_coord = y as f32 * 12.5 + self.start_y;
                    // let pixel = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), graphics::Rect::new(x_coord, y_coord, self.height, self.width), pixel_color)?;
                    // self.pixels.push(pixel);
                    let top_left_point = nalgebra::Point2::new(x_coord, y_coord);
                    let top_right_point = nalgebra::Point2::new(x_coord + 12.5, y_coord);
                    let bottom_left_point = nalgebra::Point2::new(x_coord, y_coord + 12.5);
                    let bottom_right_point = nalgebra::Point2::new(x_coord + 12.5, y_coord + 12.5);
                    screen.polygon(graphics::DrawMode::fill(), &[top_left_point, bottom_left_point, bottom_right_point, top_right_point], pixel_color).unwrap();
                }
            }
    
            graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());
            let mesh = screen.build(ctx)?;
            match graphics::draw(ctx, &mesh, graphics::DrawParam::new()){
                Ok(_) => (),
                Err(e) => println!("Error {}", e),
            };

            // for pixel in self.pixels.iter(){
            //     graphics::draw(ctx, pixel, graphics::DrawParam::default())?;
            // }
    
            graphics::present(ctx)?;
            // self.pixels.clear();
            
            self.chip_8.disable_draw_enabled();

        }


        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, _keycode: KeyCode, _keymod: KeyMods,  _repeat: bool){
        match _keycode{
            KeyCode::Key1   => self.chip_8.keyboard[0]  = true,
            KeyCode::Key2   => self.chip_8.keyboard[1]  = true,
            KeyCode::Key3   => self.chip_8.keyboard[2]  = true,
            KeyCode::Key4   => self.chip_8.keyboard[3]  = true,
            KeyCode::Q      => self.chip_8.keyboard[4]  = true,
            KeyCode::W      => self.chip_8.keyboard[5]  = true,
            KeyCode::E      => self.chip_8.keyboard[6]  = true,
            KeyCode::R      => self.chip_8.keyboard[7]  = true,
            KeyCode::A      => self.chip_8.keyboard[8]  = true,
            KeyCode::S      => self.chip_8.keyboard[9]  = true,
            KeyCode::D      => self.chip_8.keyboard[10] = true,
            KeyCode::F      => self.chip_8.keyboard[11] = true,
            KeyCode::Z      => self.chip_8.keyboard[12] = true,
            KeyCode::X      => self.chip_8.keyboard[13] = true,
            KeyCode::C      => self.chip_8.keyboard[14] = true,
            KeyCode::V      => self.chip_8.keyboard[15] = true,
            _               => (),
        };
    }

    fn key_up_event(&mut self, _ctx: &mut Context, _keycode: KeyCode, _keymod: KeyMods){
        match _keycode{
            KeyCode::Key1   => self.chip_8.keyboard[0]  = false,
            KeyCode::Key2   => self.chip_8.keyboard[1]  = false,
            KeyCode::Key3   => self.chip_8.keyboard[2]  = false,
            KeyCode::Key4   => self.chip_8.keyboard[3]  = false,
            KeyCode::Q      => self.chip_8.keyboard[4]  = false,
            KeyCode::W      => self.chip_8.keyboard[5]  = false,
            KeyCode::E      => self.chip_8.keyboard[6]  = false,
            KeyCode::R      => self.chip_8.keyboard[7]  = false,
            KeyCode::A      => self.chip_8.keyboard[8]  = false,
            KeyCode::S      => self.chip_8.keyboard[9]  = false,
            KeyCode::D      => self.chip_8.keyboard[10] = false,
            KeyCode::F      => self.chip_8.keyboard[11] = false,
            KeyCode::Z      => self.chip_8.keyboard[12] = false,
            KeyCode::X      => self.chip_8.keyboard[13] = false,
            KeyCode::C      => self.chip_8.keyboard[14] = false,
            KeyCode::V      => self.chip_8.keyboard[15] = false,
            _               => (),
        };
    }

}

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("super_simple", "ggez").window_setup(ggez::conf::WindowSetup::default());
    let (ctx, event_loop) = &mut cb.build()?;
    let state = &mut MainState::new()?;
    event::run(ctx, event_loop, state)
}
