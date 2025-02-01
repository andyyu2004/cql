use std::process::Command;

use anyhow::Result;
use reedline::{
    default_vi_insert_keybindings, default_vi_normal_keybindings, DefaultPrompt,
    DefaultPromptSegment, FileBackedHistory, KeyCode, KeyModifiers, Reedline, ReedlineEvent, Vi,
};

use scylla::Session;

use crate::{exec, ExecArgs, Format};

pub async fn run(sess: &Session) -> Result<()> {
    let tmp = tempfile::NamedTempFile::new()?;

    let history_path = match dirs::home_dir() {
        Some(home) => home.join("local/share/cql/history"),
        None => std::env::current_dir()?.join(".cql_history"),
    };

    let mut normal = default_vi_normal_keybindings();
    normal.add_binding(
        KeyModifiers::NONE,
        KeyCode::Char('v'),
        ReedlineEvent::OpenEditor,
    );

    let mut readline = Reedline::create()
        .with_history(Box::new(FileBackedHistory::with_file(
            10_000,
            history_path,
        )?))
        .with_edit_mode(Box::new(Vi::new(default_vi_insert_keybindings(), normal)));

    if let Some(cmd) = std::env::var("EDITOR").ok().map(Command::new) {
        readline = readline.with_buffer_editor(cmd, tmp.path().to_path_buf())
    }

    let prompt = DefaultPrompt::new(
        DefaultPromptSegment::Basic("cql".to_string()),
        DefaultPromptSegment::Empty,
    );

    let mut exec_args = ExecArgs {
        command: String::new(),
        flatten: false,
        output: Format::JsonPretty,
    };

    loop {
        match readline.read_line(&prompt)? {
            reedline::Signal::Success(command) => {
                exec_args.command = command;
                match exec(sess, &exec_args).await {
                    Ok(()) => (),
                    Err(err) => eprintln!("{err}"),
                }
            }
            reedline::Signal::CtrlC => continue,
            reedline::Signal::CtrlD => break,
        };
    }

    Ok(())
}
