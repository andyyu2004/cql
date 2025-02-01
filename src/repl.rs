use std::process::Command;

use anyhow::Result;
use reedline::{
    default_vi_insert_keybindings, default_vi_normal_keybindings, ColumnarMenu, DefaultCompleter,
    DefaultHinter, DefaultPrompt, DefaultPromptSegment, FileBackedHistory, KeyCode, KeyModifiers,
    MenuBuilder, Reedline, ReedlineEvent, ReedlineMenu, Vi,
};

use scylla::Session;

use crate::{exec, ExecArgs, Format};

const KWS: [&str; 114] = [
    "SELECT",
    "FROM",
    "WHERE",
    "AND",
    "OR",
    "INSERT",
    "INTO",
    "VALUES",
    "UPDATE",
    "SET",
    "DELETE",
    "CREATE",
    "TABLE",
    "KEYSPACE",
    "WITH",
    "IF",
    "EXISTS",
    "DROP",
    "ALTER",
    "ADD",
    "COLUMN",
    "PRIMARY",
    "KEY",
    "ASC",
    "DESC",
    "LIMIT",
    "ALLOW",
    "FILTERING",
    "ORDER",
    "BY",
    "ASC",
    "DESC",
    "TOKEN",
    "IN",
    "CONTAINS",
    "MAP",
    "LIST",
    "SET",
    "TUPLE",
    "BOOLEAN",
    "INT",
    "BIGINT",
    "FLOAT",
    "DOUBLE",
    "DECIMAL",
    "ASCII",
    "TEXT",
    "BLOB",
    "TIMESTAMP",
    "TIMEUUID",
    "UUID",
    "BOOLEAN",
    "VARINT",
    "INET",
    "COUNTER",
    "DATE",
    "DURATION",
    "TIME",
    "TINYINT",
    "SMALLINT",
    "DATE",
    "TIME",
    "DURATION",
    "TIMESTAMP",
    "BIGINT",
    "COUNTER",
    "DECIMAL",
    "DOUBLE",
    "FLOAT",
    "INT",
    "SMALLINT",
    "TINYINT",
    "TEXT",
    "ASCII",
    "BLOB",
    "BOOLEAN",
    "DATE",
    "DURATION",
    "TIME",
    "TIMESTAMP",
    "TIMEUUID",
    "UUID",
    "VARINT",
    "INET",
    "LIST",
    "MAP",
    "SET",
    "TUPLE",
    "PRIMARY",
    "KEY",
    "CLUSTERING",
    "ORDER",
    "BY",
    "ASC",
    "DESC",
    "ALLOW",
    "FILTERING",
    "IF",
    "EXISTS",
    "KEYSPACE",
    "CREATE",
    "DROP",
    "ALTER",
    "TABLE",
    "INDEX",
    "MATERIALIZED",
    "VIEW",
    "TYPE",
    "FUNCTION",
    "AGGREGATE",
    "OR",
    "AND",
    "NOT",
    "IN",
];

pub async fn run(sess: &Session) -> Result<()> {
    let rows = sess.query_unpaged("DESCRIBE tables", &()).await?;
    #[derive(Debug, scylla::FromRow)]
    struct Row {
        keyspace_name: String,
        #[allow(unused)]
        r#type: String,
        name: String,
    }

    let tables = rows
        .rows_typed_or_empty::<Row>()
        .filter_map(|row| {
            let row = row.ok()?;
            if row.keyspace_name.starts_with("system") {
                return None;
            }
            Some(format!("{}.{}", row.keyspace_name, row.name))
        })
        .collect::<Vec<_>>();

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

    let mut insert = default_vi_insert_keybindings();
    insert.add_binding(
        KeyModifiers::CONTROL,
        KeyCode::Char('f'),
        ReedlineEvent::HistoryHintComplete,
    );
    insert.add_binding(
        KeyModifiers::NONE,
        KeyCode::Tab,
        ReedlineEvent::UntilFound(vec![
            ReedlineEvent::Menu(MENU_NAME.to_string()),
            ReedlineEvent::MenuNext,
        ]),
    );

    const MENU_NAME: &str = "completion_menu";
    let mut completer = DefaultCompleter::with_inclusions(&['.', '_']).set_min_word_len(2);
    completer.insert(
        KWS.into_iter()
            .map(|kw| kw.to_lowercase())
            .chain(KWS.into_iter().map(|kw| kw.to_string()))
            .chain(tables)
            .collect(),
    );

    let completion_menu = Box::new(ColumnarMenu::default().with_name("completion_menu"));

    let mut readline = Reedline::create()
        .with_history(Box::new(FileBackedHistory::with_file(
            10_000,
            history_path,
        )?))
        .with_completer(Box::new(completer))
        .with_menu(ReedlineMenu::EngineCompleter(completion_menu))
        .with_quick_completions(true)
        .with_partial_completions(true)
        .with_hinter(Box::new(DefaultHinter::default()))
        .with_edit_mode(Box::new(Vi::new(insert, normal)));

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
