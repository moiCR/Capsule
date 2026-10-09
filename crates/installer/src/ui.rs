use crate::app::{App, Step};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, Gauge, List, ListItem, Paragraph, Wrap,
    },
    Frame,
};

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Header
            Constraint::Length(3), // Stepper
            Constraint::Min(12),   // Main content
            Constraint::Length(3), // Footer
        ])
        .split(f.area());

    render_header(f, chunks[0]);
    render_stepper(f, chunks[1], app.step);

    match app.step {
        Step::Welcome => render_welcome(f, chunks[2], app),
        Step::Dependencies => render_dependencies(f, chunks[2], app),
        Step::Installing => render_installing(f, chunks[2], app),
        Step::Finished => render_finished(f, chunks[2], app),
    }

    render_footer(f, chunks[3], app);
    render_password_modal(f, app);
}

fn render_header(f: &mut Frame, area: Rect) {
    let title = Line::from(vec![
        Span::styled("◆ CAPSULE ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("INSTALLER ◆", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
    ]);
    let subtitle = Line::from(vec![
        Span::styled(
            "Next-Gen Dynamic Island Shell for Linux",
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let p = Paragraph::new(vec![title, subtitle])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan)),
        );
    f.render_widget(p, area);
}

fn render_stepper(f: &mut Frame, area: Rect, current: Step) {
    let steps = [
        ("1. Welcome", Step::Welcome),
        ("2. Dependencies", Step::Dependencies),
        ("3. Installing", Step::Installing),
        ("4. Finished", Step::Finished),
    ];

    let mut spans = Vec::new();
    for (i, (name, s)) in steps.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" ── ", Style::default().fg(Color::DarkGray)));
        }

        let is_current = *s == current;
        let is_done = match (*s, current) {
            (Step::Welcome, Step::Dependencies | Step::Installing | Step::Finished) => true,
            (Step::Dependencies, Step::Installing | Step::Finished) => true,
            (Step::Installing, Step::Finished) => true,
            _ => false,
        };

        if is_current {
            spans.push(Span::styled(
                format!(" [{}] ", name),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED),
            ));
        } else if is_done {
            spans.push(Span::styled(
                format!(" ✔ {} ", name),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(
                format!(" {} ", name),
                Style::default().fg(Color::Gray),
            ));
        }
    }

    let p = Paragraph::new(Line::from(spans))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
    f.render_widget(p, area);
}

fn render_welcome(f: &mut Frame, area: Rect, app: &App) {
    let sub = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(6)])
        .split(area);

    let helper_name = app
        .aur_helper
        .as_ref()
        .map(|h| h.name())
        .unwrap_or("None (Unsupported distro)");

    let cur_ver = app.installed_version.as_deref().unwrap_or("Not installed");
    let target_ver = app.target_version.as_deref().unwrap_or("Detecting...");
    let channel_str = if app.beta_mode { "Beta (Pre-release)" } else { "Stable (Latest)" };
    let running_str = if app.capsule_was_running {
        "Running (will restart on update)"
    } else {
        "Stopped"
    };

    let info_lines = vec![
        Line::from(vec![
            Span::styled("System Target:       ", Style::default().fg(Color::Gray)),
            Span::styled(app.target, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Package Manager:     ", Style::default().fg(Color::Gray)),
            Span::styled(helper_name, Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled("Installed Version:   ", Style::default().fg(Color::Gray)),
            Span::styled(cur_ver, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("Target Version:      ", Style::default().fg(Color::Gray)),
            Span::styled(target_ver, Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" [{}]", channel_str), Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("Capsule Status:      ", Style::default().fg(Color::Gray)),
            Span::styled(running_str, Style::default().fg(Color::Blue)),
        ]),
        Line::from(vec![
            Span::styled("Wallpaper Daemon:    ", Style::default().fg(Color::Gray)),
            Span::styled(
                app.wallpaper_daemon.as_deref().unwrap_or("None (awww / swww)"),
                if app.wallpaper_daemon.is_some() {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Yellow)
                },
            ),
        ]),
    ];

    let info_block = Paragraph::new(info_lines).block(
        Block::default()
            .title(" System Detection ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(info_block, sub[0]);

    let options = [
        "▶ Continue to Dependencies Check",
        "⚡ Toggle Release Channel (Stable / Beta)",
        "✕ Exit Installer",
    ];

    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, &opt)| {
            if i == app.selected_index {
                ListItem::new(Span::styled(
                    format!("  {}  ", opt),
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
            } else {
                ListItem::new(Span::styled(
                    format!("    {}", opt),
                    Style::default().fg(Color::White),
                ))
            }
        })
        .collect();

    let menu = List::new(items).block(
        Block::default()
            .title(" Actions ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(menu, sub[1]);
}

fn render_dependencies(f: &mut Frame, area: Rect, app: &App) {
    let sub = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(6)])
        .split(area);

    let half = (app.dependencies.len() + 1) / 2;
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(sub[0]);

    let make_list = |slice: &[(String, bool)]| -> Vec<ListItem<'static>> {
        slice
            .iter()
            .map(|(pkg, inst)| {
                let pkg_name = pkg.clone();
                if *inst {
                    ListItem::new(Line::from(vec![
                        Span::styled(" ✔ ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                        Span::styled(pkg_name, Style::default().fg(Color::White)),
                        Span::styled(" (installed)", Style::default().fg(Color::DarkGray)),
                    ]))
                } else {
                    ListItem::new(Line::from(vec![
                        Span::styled(" ➜ ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                        Span::styled(pkg_name, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                        Span::styled(" (missing)", Style::default().fg(Color::Red)),
                    ]))
                }
            })
            .collect()
    };

    let col1_items = make_list(&app.dependencies[..half]);
    let col2_items = make_list(&app.dependencies[half..]);

    let missing = app.missing_dependencies_count();
    let title = if missing == 0 {
        " Dependencies (All 20 installed ✔) ".to_string()
    } else {
        format!(" Dependencies ({} missing) ", missing)
    };

    let list1 = List::new(col1_items).block(
        Block::default()
            .title(title)
            .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
            .border_type(BorderType::Rounded)
            .border_style(if missing > 0 { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::Green) }),
    );
    let list2 = List::new(col2_items).block(
        Block::default()
            .borders(Borders::TOP | Borders::RIGHT | Borders::BOTTOM)
            .border_type(BorderType::Rounded)
            .border_style(if missing > 0 { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::Green) }),
    );

    f.render_widget(list1, cols[0]);
    f.render_widget(list2, cols[1]);

    let options = [
        if missing > 0 {
            "⚡ Install Missing Dependencies via Package Manager"
        } else {
            "✔ Re-check Dependencies"
        },
        "▶ Proceed to Install Capsule Binary",
        "◀ Back to Welcome Screen",
    ];

    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, &opt)| {
            if i == app.selected_index {
                ListItem::new(Span::styled(
                    format!("  {}  ", opt),
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
            } else {
                ListItem::new(Span::styled(
                    format!("    {}", opt),
                    Style::default().fg(Color::White),
                ))
            }
        })
        .collect();

    let menu = List::new(items).block(
        Block::default()
            .title(" Actions ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(menu, sub[1]);
}

fn render_installing(f: &mut Frame, area: Rect, app: &App) {
    let sub = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(6)])
        .split(area);

    let pct = (app.progress.clamp(0.0, 1.0) * 100.0) as u16;
    let gauge = Gauge::default()
        .block(
            Block::default()
                .title(format!(" Progress: {} ", app.status_message))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .gauge_style(
            Style::default()
                .fg(Color::Cyan)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .percent(pct);

    f.render_widget(gauge, sub[0]);

    let log_items: Vec<ListItem> = app
        .logs
        .iter()
        .rev()
        .take(15)
        .rev()
        .map(|line| {
            ListItem::new(Line::from(vec![
                Span::styled("  ➜ ", Style::default().fg(Color::DarkGray)),
                Span::styled(line, Style::default().fg(Color::Gray)),
            ]))
        })
        .collect();

    let logs_widget = List::new(log_items).block(
        Block::default()
            .title(" Activity Log ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(logs_widget, sub[1]);
}

fn render_finished(f: &mut Frame, area: Rect, app: &App) {
    let sub = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(6)])
        .split(area);

    let ver = app.installed_version.as_deref().unwrap_or("latest");
    let launch_check = if app.launch_on_finish { "[x]" } else { "[ ]" };

    let text = vec![
        Line::from(vec![
            Span::styled("✔ ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("Capsule {} successfully installed to /usr/local/bin/capsule!", ver),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Desktop entry created at: ", Style::default().fg(Color::DarkGray)),
            Span::styled("/usr/share/applications/capsule.desktop", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(Span::styled("Autostart Setup:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(vec![
            Span::styled("  • Hyprland: ", Style::default().fg(Color::Gray)),
            Span::styled("exec-once = capsule", Style::default().fg(Color::Cyan)),
            Span::styled(" in ~/.config/hypr/hyprland.conf", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("  • Niri:     ", Style::default().fg(Color::Gray)),
            Span::styled("spawn-at-startup \"capsule\"", Style::default().fg(Color::Cyan)),
            Span::styled(" in ~/.config/niri/config.kdl", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let p = Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(" Installation Succeeded ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Green)),
        );
    f.render_widget(p, sub[0]);

    let options = [
        format!("{} Launch Capsule daemon on exit", launch_check),
        "▶ Finish and Exit".to_string(),
    ];

    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, opt)| {
            if i == app.selected_index {
                ListItem::new(Span::styled(
                    format!("  {}  ", opt),
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ))
            } else {
                ListItem::new(Span::styled(
                    format!("    {}", opt),
                    Style::default().fg(Color::White),
                ))
            }
        })
        .collect();

    let menu = List::new(items).block(
        Block::default()
            .title(" Next Steps ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(menu, sub[1]);
}

fn render_footer(f: &mut Frame, area: Rect, app: &App) {
    let key_hints = match app.step {
        Step::Welcome => vec![
            Span::styled("[↑/↓] ", Style::default().fg(Color::Cyan)),
            Span::styled("Navigate  ", Style::default().fg(Color::Gray)),
            Span::styled("[Enter] ", Style::default().fg(Color::Cyan)),
            Span::styled("Select  ", Style::default().fg(Color::Gray)),
            Span::styled("[b] ", Style::default().fg(Color::Cyan)),
            Span::styled("Toggle Beta  ", Style::default().fg(Color::Gray)),
            Span::styled("[q/Esc] ", Style::default().fg(Color::Cyan)),
            Span::styled("Quit", Style::default().fg(Color::Gray)),
        ],
        Step::Dependencies => vec![
            Span::styled("[↑/↓] ", Style::default().fg(Color::Cyan)),
            Span::styled("Navigate  ", Style::default().fg(Color::Gray)),
            Span::styled("[Enter] ", Style::default().fg(Color::Cyan)),
            Span::styled("Select  ", Style::default().fg(Color::Gray)),
            Span::styled("[Esc] ", Style::default().fg(Color::Cyan)),
            Span::styled("Back", Style::default().fg(Color::Gray)),
        ],
        Step::Installing => vec![
            Span::styled("Installing Capsule... Please wait.", Style::default().fg(Color::Cyan)),
        ],
        Step::Finished => vec![
            Span::styled("[↑/↓] ", Style::default().fg(Color::Cyan)),
            Span::styled("Navigate  ", Style::default().fg(Color::Gray)),
            Span::styled("[Space] ", Style::default().fg(Color::Cyan)),
            Span::styled("Toggle Launch  ", Style::default().fg(Color::Gray)),
            Span::styled("[Enter/q] ", Style::default().fg(Color::Green)),
            Span::styled("Done", Style::default().fg(Color::Gray)),
        ],
    };

    let p = Paragraph::new(Line::from(key_hints))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
    f.render_widget(p, area);
}

fn render_password_modal(f: &mut Frame, app: &App) {
    if !app.show_password_modal {
        return;
    }

    let area = centered_rect(65, 30, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Sudo Authentication Required ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Prompt
            Constraint::Length(3), // Input
            Constraint::Length(2), // Error
            Constraint::Length(1), // Hint
        ])
        .margin(2)
        .split(area);

    let prompt = Paragraph::new("Enter user password to authorize system installation (/usr/local/bin):")
        .style(Style::default().fg(Color::White));
    f.render_widget(prompt, chunks[0]);

    let masked: String = "•".repeat(app.password_input.len());
    let input = Paragraph::new(format!("  {}  ", masked))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    f.render_widget(input, chunks[1]);

    if let Some(err) = &app.password_error {
        let err_widget = Paragraph::new(Span::styled(
            err,
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ));
        f.render_widget(err_widget, chunks[2]);
    }

    let hint = Paragraph::new("[Enter] Confirm  [Esc] Cancel")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(hint, chunks[3]);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
