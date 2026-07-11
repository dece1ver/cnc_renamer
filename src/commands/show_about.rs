use crate::error::AppResult;
use crate::ui::return_back;
use crossterm::{
    execute,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
};
use std::io::stdout;

/// Описание поддерживаемой системы ЧПУ и формата её имени.
struct CncSystem {
    name: &'static str,
    format: &'static str,
    exts: &'static str,
}

/// Список поддерживаемых систем ЧПУ.
const CNC_SYSTEMS: &[CncSystem] = &[
    CncSystem {
        name: "Fanuc 0i",
        format: "O0001(НАЗВАНИЕ)",
        exts: ".nc, .cnc, ...",
    },
    CncSystem {
        name: "Fanuc 0i-*F",
        format: "<НАЗВАНИЕ>",
        exts: ".nc, .cnc, ...",
    },
    CncSystem {
        name: "Mazatrol Smart",
        format: "имя по смещению (байт 80)",
        exts: ".pbg, .pbd",
    },
    CncSystem {
        name: "Sinumerik 840D sl",
        format: "MSG(\"НАЗВАНИЕ\")",
        exts: ".mpf, .spf",
    },
    CncSystem {
        name: "Heidenhain",
        format: "BEGIN PGM НАЗВАНИЕ MM",
        exts: ".h",
    },
];

/// Показывает экран «О программе» с версией и списком поддерживаемых форматов.
pub fn show_about() -> AppResult<()> {
    clearscreen::clear()?;

    execute!(
        stdout(),
        SetAttribute(Attribute::Framed),
        SetForegroundColor(Color::Green),
        Print("CNC Remedy"),
        ResetColor,
        SetAttribute(Attribute::Reset),
        Print("\n"),
        SetForegroundColor(Color::DarkGrey),
        Print("Утилита для работы с файлами управляющих программ ЧПУ.\n"),
        ResetColor,
    )?;

    execute!(
        stdout(),
        Print("\n"),
        SetAttribute(Attribute::Underlined),
        Print("Команды контекстного меню"),
        SetAttribute(Attribute::Reset),
        Print("\n"),
    )?;

    execute!(
        stdout(),
        SetForegroundColor(Color::DarkGrey),
        Print("  Доступные команды:\n"),
        ResetColor,
    )?;

    let commands = [
        (
            "Переименовать",
            "ПКМ по файлу/папке",
            "Переименовывает файл по названию УП внутри (prefix/suffix/overwrite)",
        ),
        (
            "Архивировать",
            "ПКМ по файлу",
            "Перемещает в _/дата/ (формат и источник времени настраиваются)",
        ),
        (
            "Очистить комментарии",
            "ПКМ по файлу",
            "Удаляет строки комментариев (символы и keep_string настраиваются)",
        ),
        (
            "Список инструмента",
            "ПКМ по файлу",
            "Генерирует таблицу инструментов из G-кода",
        ),
    ];

    for (name, usage, desc) in commands {
        execute!(
            stdout(),
            SetForegroundColor(Color::Yellow),
            Print(format!("  {:<22}", name)),
            SetForegroundColor(Color::White),
            Print(format!("{}\n", usage)),
            SetForegroundColor(Color::DarkGrey),
            Print(format!("  {:<22}{}\n", "", desc)),
            ResetColor,
        )?;
    }

    execute!(
        stdout(),
        Print("\n"),
        SetAttribute(Attribute::Underlined),
        Print("Параметры настроек (cncr.toml)"),
        SetAttribute(Attribute::Reset),
        Print("\n"),
    )?;

    let extra_params = [
        (
            "Переименовать",
            "prefix / suffix",
            "добавляется к имени файла",
        ),
        (
            "",
            "overwrite = true",
            "при конфликте остаётся новейший файл",
        ),
        ("", "recurse = true", "обработка поддиректорий (background)"),
        (
            "Архивировать",
            "date_format",
            "формат даты подпапки (%d%m%y.%H%M)",
        ),
        ("", "timestamp_source", "now / modified (дата изменения)"),
        (
            "Очистить",
            "comment_chars",
            "символы комментариев (; по умолч.)",
        ),
        ("", "keep_string", "не удалять строки с фрагментом"),
        ("", "strip_mode", "starts-with / contains (где искать символ)"),
        (
            "Список инстр.",
            "fanuc_milling_line",
            "строка вставки таблицы (Fanuc фрезер)",
        ),
        ("", "fanuc_lathe_line", "строка вставки таблицы (Fanuc токар.)"),
        ("", "sinumerik_line", "строка вставки таблицы (Sinumerik)"),
        ("", "heidenhain_line", "строка вставки таблицы (Heidenhain)"),
        (
            "",
            "fanuc_milling_print_tool_number",
            "выводить номер/H/D (Fanuc фрезер)",
        ),
        (
            "",
            "fanuc_lathe_print_tool_number",
            "выводить номер (Fanuc токар.)",
        ),
        ("", "sinumerik_print_tool_number", "выводить номер (Sinumerik)"),
        (
            "",
            "heidenhain_print_tool_number",
            "выводить номер (Heidenhain)",
        ),
    ];

    for (cmd, param, desc) in extra_params {
        execute!(
            stdout(),
            SetForegroundColor(Color::Yellow),
            Print(format!("  {:<18}", cmd)),
            SetForegroundColor(Color::White),
            Print(format!("{:<22}", param)),
            SetForegroundColor(Color::DarkGrey),
            Print(desc),
            ResetColor,
            Print("\n"),
        )?;
    }

    execute!(
        stdout(),
        Print("\n"),
        SetAttribute(Attribute::Underlined),
        Print("Поддерживаемые СЧПУ"),
        SetAttribute(Attribute::Reset),
        Print("\n"),
    )?;

    for sys in CNC_SYSTEMS {
        execute!(
            stdout(),
            SetForegroundColor(Color::Yellow),
            Print("  • "),
            SetForegroundColor(Color::White),
            Print(format!("{:<38}", sys.name)),
        )?;
        let fmt = sys.format;
        if let Some(pos) = fmt.find("НАЗВАНИЕ") {
            execute!(
                stdout(),
                SetForegroundColor(Color::DarkGrey),
                Print(&fmt[..pos]),
                SetForegroundColor(Color::Cyan),
                Print("НАЗВАНИЕ"),
                SetForegroundColor(Color::DarkGrey),
                Print(&fmt[pos + "НАЗВАНИЕ".len()..]),
                ResetColor,
            )?;
        } else {
            execute!(
                stdout(),
                SetForegroundColor(Color::DarkGrey),
                Print(fmt),
                ResetColor,
            )?;
        }
        execute!(
            stdout(),
            SetForegroundColor(Color::DarkGrey),
            Print(format!(" [{}]", sys.exts)),
            ResetColor,
            Print("\n"),
        )?;
    }

    execute!(
        stdout(),
        Print("\n"),
        SetAttribute(Attribute::Underlined),
        Print("Установка"),
        SetAttribute(Attribute::Reset),
        Print("\n"),
        SetForegroundColor(Color::DarkGrey),
        Print("  Требуются права администратора. При установке:\n"),
        ResetColor,
    )?;

    let steps = [
        r#"Копируется в "C:\Program Files\dece1ver\CNC Remedy""#,
        "Добавляется подменю CNC Remedy в контекстное меню файлов, папок и фона",
        "Путь прописывается в PATH",
    ];
    for (i, step) in steps.iter().enumerate() {
        execute!(
            stdout(),
            SetForegroundColor(Color::Yellow),
            Print(format!("  {}. ", i + 1)),
            ResetColor,
            Print(format!("{}\n", step)),
        )?;
    }

    execute!(
        stdout(),
        Print("\n"),
        SetAttribute(Attribute::Underlined),
        Print("CLI"),
        SetAttribute(Attribute::Reset),
        Print("\n"),
    )?;

    let cli_flags = [
        ("cncr <команда> <файлы...>", "запуск команды для файлов"),
        ("cncr install", "установить программу"),
        ("cncr uninstall", "удалить программу"),
        ("cncr rename <файлы...>", "переименовать УП"),
        ("cncr archive <файлы...>", "архивировать УП"),
        ("cncr strip-comments <файлы...>", "очистить комментарии"),
        ("cncr generate-tool-list <файлы...>", "сформировать таблицу инструментов"),
        ("cncr --reset-config", "сбросить настройки"),
        ("cncr (без аргументов)", "интерактивное меню"),
    ];

    for (usage, desc) in cli_flags {
        execute!(
            stdout(),
            SetForegroundColor(Color::Yellow),
            Print(format!("  {:<40}", usage)),
            SetForegroundColor(Color::DarkGrey),
            Print(desc),
            ResetColor,
            Print("\n"),
        )?;
    }

    execute!(
        stdout(),
        Print("\n"),
        SetForegroundColor(Color::DarkGrey),
        Print("Поведение команд (символы комментариев, обработка конфликтов,\n"),
        Print("рекурсивный обход и т.д.) задаётся в конфигурации.\n"),
        ResetColor,
    )?;

    return_back()?;
    Ok(())
}
