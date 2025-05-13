// src/shell.rs
use alloc::{string::ToString, string::String, vec::Vec};
use crate::{
    keyboard::{self, Key, KeyState},
    vga::{vga_print_char, vga_print, vga_clear_screen, vga_set_foreground, VgaTextModeColor}
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
            Key::Up => self.handle_up_arrow(),
            Key::Down => self.handle_down_arrow(),
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

    fn handle_up_arrow(&mut self) {
        if !self.command_history.is_empty() {
            if self.history_index > 0 {
                // Clear current line
                self.clear_current_line();
                
                // Move to previous command in history
                self.history_index -= 1;
                let cmd = &self.command_history[self.history_index];
                
                // Set buffer and display the command
                self.buffer = cmd.as_bytes().to_vec();
                vga_print(&self.buffer);
            }
        }
    }

    fn handle_down_arrow(&mut self) {
        if !self.command_history.is_empty() {
            if self.history_index < self.command_history.len() - 1 {
                // Clear current line
                self.clear_current_line();
                
                // Move to next command in history
                self.history_index += 1;
                let cmd = &self.command_history[self.history_index];
                
                // Set buffer and display the command
                self.buffer = cmd.as_bytes().to_vec();
                vga_print(&self.buffer);
            } else if self.history_index == self.command_history.len() - 1 {
                // Clear the line if we're at the end of history
                self.clear_current_line();
                self.history_index = self.command_history.len();
                self.buffer.clear();
            }
        }
    }

    fn clear_current_line(&mut self) {
        for _ in 0..self.buffer.len() {
            vga_print_char(0x08); // Backspace
        }
        for _ in 0..self.buffer.len() {
            vga_print_char(b' ');
        }
        for _ in 0..self.buffer.len() {
            vga_print_char(0x08);
        }
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
            "clear" => self.clear_screen(),
            "poweroff" => self.poweroff(),
            cmd if cmd.starts_with("echo ") => self.echo(&cmd[5..]),
            _ => self.unknown_command(cmd),
        }
    }

    fn poweroff(&self) {
        vga_print(b"Shutting down...\n");
        
        // Method 1: QEMU shutdown with exit code
        unsafe {
            // 0x31 is the magic exit code for QEMU's isa-debug-exit device
            // Writing to port 0xf4 will make QEMU exit with status (0x31 << 1) | 1 = 99
            x86_64::instructions::port::Port::new(0xf4).write(0x31 as u8);
        }
        
        // If QEMU didn't exit, try ACPI shutdown
        unsafe {
            for port in [0x604, 0xB004].iter() {  // Try different ACPI ports
                x86_64::instructions::port::Port::new(*port).write(0x2000 as u16);
            }
        }
        
        // If still running, halt the CPU
        loop {
            x86_64::instructions::hlt();
        }
    }

    fn show_help(&self) {
        vga_print(b"Available commands:\n");
        vga_print(b"- help: Show this help\n");
        vga_print(b"- echo <text>: Print text\n");
        vga_print(b"- clear: Clear screen\n");
        //TODO: add poweroff if works
    }

    fn clear_screen(&self) {
        vga_clear_screen();
        vga_set_foreground(VgaTextModeColor::LightBlue);
        vga_print(b"
           _       _                 
 _ __ ___ (_)_ __ | | __    ___  ___ 
| '_ ` _ \\| | '_ \\| |/ /   / _ \\/ __|
| | | | | | | | | |   <   | (_) \\__ \\
|_| |_| |_|_|_| |_|_|\\_\\___\\___/|___/
                      |_____|        
\n");
        vga_set_foreground(VgaTextModeColor::White);
        vga_print(b"\n");
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
