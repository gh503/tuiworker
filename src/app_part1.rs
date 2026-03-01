APP_PART1
cat app.rs | sed -n '64,71p' > app_terminal_impl.txt

cat >> app_terminal_impl.txt << 'TERMINAL_IMPL'
impl TerminalTab {
    pub fn init_pty(&mut self, shell: String) -> Result<(), String> {
        if self.pty_session.is_some() {
            return Ok(());
        }
        let mut session = PtySession::new(shell);
        session.start()?;
        self.pty_session = Some(session);
        Ok(())
    }

    pub fn execute(&mut self, command: String) {
        if command.trim().is_empty() { return; }
        if let Some(ref mut pty) = self.pty_session {
            let full_command = format!("{}\nexit\n", command);
            for c in full_command.chars() { let _ = pty.send_char(c); }
            self.command_execution_active = true;
        }
    }

    pub fn update_output(&mut self) {
        if let Some(ref pty) = self.pty_session {
            let output = pty.get_output();
            self.command_output_buffer = output;
        }
    }

    pub fn get_output(&self) -> String {
        self.command_output_buffer.clone()
    }

    pub fn close(&mut self) {
        if let Some(mut session) = self.pty_session.take() {
            session.stop();
        }
        self.is_active = false;
    }
}
TERMINAL_IMPL

# 添加到文件末尾
cat terminal_impl.txt >> app.rs
rm terminal_impl.txt
