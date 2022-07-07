use ncurses::*;

#[allow(dead_code)]
enum BranchType {
    Trunk,
    ShootLeft,
    ShootRight,
    Dying,
    Dead
}

#[allow(dead_code)]
struct Config {
    live: u32,
    infinite: u32,
    screensaver: u32,
    print_tree: u32,
    verbosity: u32,
    life_start: u32,
    multiplier: u32,
    base_type: u32,
    seed: u32,
    leaves_size: u32,
    save: u32,
    load: u32,
    target_branch_count: u32,

    time_wait: f32,
    time_step: f32,

    //message: &char,
    //leaves: [&char; 64],
    //save_file: &char,
    //load_file: &char
}

#[allow(dead_code)]
struct NcursesObjects {
    base_win: WINDOW,
    tree_win: WINDOW,
    message_border_win: WINDOW,
    message_win: WINDOW,

    base_panel: PANEL,
    tree_panel: PANEL,
    message_border_panel: PANEL,
    message_panel: PANEL
}

#[allow(dead_code)]
struct Counters {
    branches: u32,
    shoots: u32,
    shoot_counter: u32
}

fn main() {
    initscr();

    addstr("Hello, World!");

    getch();

    endwin();
}
