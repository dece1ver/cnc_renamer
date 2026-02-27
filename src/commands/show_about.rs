use crate::ui::return_back;
use crossterm::{
    execute,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
};
use std::io::stdout;

struct CncSystem {
    name: &'static str,
    format: &'static str,
}

const CNC_SYSTEMS: &[CncSystem] = &[
    CncSystem { name: "Fanuc 0i",          format: "O0001(НАЗВАНИЕ)"       },
    CncSystem { name: "Fanuc 0i-*F",       format: "<НАЗВАНИЕ>"            },
    CncSystem { name: "Mazatrol Smart",    format: ".PBG / .PBD"           },
    CncSystem { name: "Sinumerik 840D sl", format: "MSG(\"НАЗВАНИЕ\")"     },
    CncSystem { name: "Heidenhain",        format: "BEGIN PGM НАЗВАНИЕ MM" },
];

pub fn show_about() {
    clearscreen::clear().unwrap();

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
    ).unwrap();

    // Команды
    execute!(
        stdout(),
        Print("\n"),
        SetAttribute(Attribute::Underlined),
        Print("Команды"),
        SetAttribute(Attribute::Reset),
        Print("\n"),
    ).unwrap();

    let commands = [
        (
            "Переименовать",
            "ПКМ по файлу/папке или cncr <путь>",
            "Переименовывает файл по названию УП внутри него",
        ),
        (
            "Архивировать",
            "ПКМ по файлу или cncr <путь> -arc",
            "Перемещает файл в папку _ рядом с ним, в подпапку с датой",
        ),
    ];

    for (name, usage, desc) in commands {
        execute!(
            stdout(),
            SetForegroundColor(Color::Yellow),
            Print(format!("  {:<16}", name)),
            SetForegroundColor(Color::White),
            Print(format!("{}\n", usage)),
            SetForegroundColor(Color::DarkGrey),
            Print(format!("  {:<16}{}\n", "", desc)),
            ResetColor,
        ).unwrap();
    }

    // Поддерживаемые СЧПУ
    execute!(
        stdout(),
        Print("\n"),
        SetAttribute(Attribute::Underlined),
        Print("Поддерживаемые СЧПУ"),
        SetAttribute(Attribute::Reset),
        Print("\n"),
    ).unwrap();

    for sys in CNC_SYSTEMS {
        execute!(
            stdout(),
            SetForegroundColor(Color::Yellow),
            Print("  • "),
            SetForegroundColor(Color::White),
            Print(format!("{:<20}", sys.name)),
            SetForegroundColor(Color::DarkGrey),
            Print(sys.format),
            ResetColor,
            Print("\n"),
        ).unwrap();
    }

    // Установка
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
    ).unwrap();

    let steps = [
        r#"Копируется в "C:\Program Files\dece1ver\CNC Remedy""#,
        "Добавляется в контекстное меню файлов и папок",
        "Путь прописывается в PATH",
    ];
    for (i, step) in steps.iter().enumerate() {
        execute!(
            stdout(),
            SetForegroundColor(Color::Yellow),
            Print(format!("  {}. ", i + 1)),
            ResetColor,
            Print(format!("{}\n", step)),
        ).unwrap();
    }

    execute!(
        stdout(),
        Print("\n"),
        SetForegroundColor(Color::DarkGrey),
        Print("При обработке директории вложенные папки не затрагиваются.\n"),
        Print("Если файл с таким именем уже существует — создаётся копия с номером.\n"),
        ResetColor,
    ).unwrap();

    return_back();
}