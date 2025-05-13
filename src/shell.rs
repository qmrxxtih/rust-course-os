// src/shell.rs
use alloc::{string::ToString, string::String, vec::Vec};
use crate::{
    keyboard::{self, Key, KeyState},
    vga::{vga_print_char, vga_print, vga_clear_screen}
};

pub struct Shell {
    buffer: Vec<u8>,
    command_history: Vec<String>,
    history_index: usize,
    last_key: Option<Key>,
    repeat_counter: u8,
}

impl Shell {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            command_history: Vec::new(),
            history_index: 0,
            last_key: None,
            repeat_counter: 0,
        }
    }

    pub fn run(&mut self) {
        self.show_prompt();

        loop {
            if keyboard::has_key_event() {
                if let Some(key_event) = keyboard::translate_key() {
                    // Only process key presses (not releases)
                    if key_event.state {
                        self.process_key(key_event.key);
                    }
                }
            }
            x86_64::instructions::hlt();
        }
    }

    fn show_prompt(&self) {
        vga_print(b"$ ");
    }

    fn process_key(&mut self, key: Key) {
        match key {
            Key::Char(c) => {
                // Only process printable ASCII characters
                if c.is_ascii_graphic() || c == b' ' {
                    self.handle_char(c);
                }
            },
            Key::Backspace => self.handle_backspace(),
            Key::Enter => self.handle_enter(),
            _ => {} // Ignore other keys
        }
    }

    fn handle_char(&mut self, c: u8) {
        self.buffer.push(c);
        vga_print_char(c);
    }

    fn handle_backspace(&mut self) {
        if !self.buffer.is_empty() {
            self.buffer.pop();
            // Backspace: move cursor back, print space, move back again
            vga_print_char(0x08);  // ASCII backspace
            vga_print_char(b' ');  // Overwrite with space
            vga_print_char(0x08);  // Move cursor back again
        }
    }

    fn handle_enter(&mut self) {
        vga_print_char(b'\n');
        
        if !self.buffer.is_empty() {
            let cmd = String::from_utf8_lossy(&self.buffer).to_string();
            self.process_command(&cmd);
            self.add_to_history(cmd);
        }
        
        self.buffer.clear();
        self.show_prompt();
    }

    fn add_to_history(&mut self, cmd: String) {
        self.command_history.push(cmd);
        self.history_index = self.command_history.len();
    }

    fn process_command(&mut self, cmd: &str) {
        let cmd = cmd.trim();
        match cmd {
            "" => {},
            "help" => self.show_help(),
            "clear" => vga_clear_screen(),
            cmd if cmd.starts_with("echo ") => self.echo(&cmd[5..]),
            _ => self.unknown_command(cmd),
        }
    }

    fn show_help(&self) {
        vga_print(b"Available commands:\n");
        vga_print(b"- help: Show this help\n");
        vga_print(b"- echo <text>: Print text\n");
        vga_print(b"- clear: Clear screen\n");
    }

    fn echo(&self, text: &str) {
        vga_print(text.as_bytes());
        vga_print(b"\n");
    }

    fn unknown_command(&self, cmd: &str) {
        vga_print(b"Unknown command: ");
        vga_print(cmd.as_bytes());
        vga_print(b"\n");
    }
}
