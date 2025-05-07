use crate::shell::Shell;

use super::echo;

impl Shell {
    pub fn pwd(&self) {
        echo(format!("{}\n", self.abs_cwd).as_str());
    }
}
