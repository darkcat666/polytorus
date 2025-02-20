// src/cli/tui.rs
use crate::{
    Result,
    crypto::wallets::Wallets,
    command::cli::{cmd_create_wallet, cmd_print_chain, cmd_list_address, cmd_reindex, cmd_create_blockchain, cmd_get_balance, cmd_send},
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Terminal,
};
use std::io;
use std::time::{Duration, Instant};

pub struct TuiApp {
    pub menu_items: Vec<&'static str>,
    pub state: ListState,
}

impl TuiApp {
    pub fn new() -> Self {
        let menu_items = vec![
            "Print Chain",
            "Create Wallet",
            "List Addresses",
            "Reindex UTXO",
            "Create Blockchain",
            "Get Balance",
            "Send",
            "Start Node",
            "Start Miner",
            "Quit",
        ];
        let mut state = ListState::default();
        state.select(Some(0));
        Self { menu_items, state }
    }

    /// 次の項目へ移動
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.menu_items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    /// 前の項目へ移動
    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.menu_items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    /// 現在選択されている項目を取得
    pub fn selected_item(&self) -> Option<&&str> {
        self.state.selected().map(|i| &self.menu_items[i])
    }
}

/// TUI を開始するエントリポイント
pub fn run_tui() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    res
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    let mut app = TuiApp::new();
    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| {
            let size = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Percentage(80),
                        Constraint::Percentage(20),
                    ]
                    .as_ref(),
                )
                .split(size);

            let items: Vec<ListItem> = app
                .menu_items
                .iter()
                .map(|i| ListItem::new(*i))
                .collect();
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Menu"))
                .highlight_style(Style::default().fg(Color::Yellow))
                .highlight_symbol(">> ");
            f.render_stateful_widget(list, chunks[0], &mut app.state);

            let instructions = Block::default()
                .borders(Borders::ALL)
                .title("Instructions");
            f.render_widget(instructions, chunks[1]);
        })?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Down => app.next(),
                    KeyCode::Up => app.previous(),
                    KeyCode::Enter => {
                        if let Some(selected) = app.selected_item() {
                            // 各コマンド実行時は CLI と同じ関数を呼び出す
                            match *selected {
                                "Print Chain" => {
                                    cmd_print_chain()?;
                                }
                                "Create Wallet" => {
                                    let addr = cmd_create_wallet()?;
                                    println!("Wallet created: {}", addr);
                                }
                                "List Addresses" => {
                                    cmd_list_address()?;
                                }
                                "Reindex UTXO" => {
                                    let count = cmd_reindex()?;
                                    println!("UTXO reindexed. Transaction count: {}", count);
                                }
                                "Create Blockchain" => {
                                    let ws = Wallets::new()?;
                                    if let Some(addr) = ws.get_all_addresses().first() {
                                        cmd_create_blockchain(addr)?;
                                    } else {
                                        println!("No wallet found. Create one first.");
                                    }
                                }
                                "Get Balance" => {
                                    let ws = Wallets::new()?;
                                    if let Some(addr) = ws.get_all_addresses().first() {
                                        let balance = cmd_get_balance(addr)?;
                                        println!("Balance for {}: {}", addr, balance);
                                    } else {
                                        println!("No wallet found. Create one first.");
                                    }
                                }
                                "Send" => {
                                    // ※ Send は対話入力が必要になるため、ここでは簡易的にメッセージ表示のみ
                                    println!("Send functionality is not fully interactive in TUI yet.");
                                }
                                "Start Node" => {
                                    println!("Start Node functionality is not supported in TUI mode.");
                                }
                                "Start Miner" => {
                                    println!("Start Miner functionality is not supported in TUI mode.");
                                }
                                "Quit" => break,
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    Ok(())
}
