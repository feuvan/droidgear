use crate::app;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy)]
struct Theme {
    border: Color,
    title: Color,
    dim: Color,
    selection_bg: Color,
    selection_fg: Color,
    selection_mod: Modifier,
    success: Color,
    error: Color,
    warning: Color,
    modal_bg: Color,
}

impl Theme {
    fn plain() -> Self {
        Self {
            border: Color::Reset,
            title: Color::Reset,
            dim: Color::Reset,
            selection_bg: Color::Reset,
            selection_fg: Color::Reset,
            selection_mod: Modifier::REVERSED,
            success: Color::Reset,
            error: Color::Reset,
            warning: Color::Reset,
            modal_bg: Color::Reset,
        }
    }

    fn nord() -> Self {
        // Inspired by the Nord palette (similar vibe to many “modern” TUIs like lazygit).
        let nord0 = Color::Rgb(0x2e, 0x34, 0x40);
        let nord3 = Color::Rgb(0x4c, 0x56, 0x6a);
        let nord6 = Color::Rgb(0xec, 0xef, 0xf4);
        let nord8 = Color::Rgb(0x88, 0xc0, 0xd0);
        let nord10 = Color::Rgb(0x5e, 0x81, 0xac);
        let nord11 = Color::Rgb(0xbf, 0x61, 0x6a);
        let nord13 = Color::Rgb(0xeb, 0xcb, 0x8b);
        let nord14 = Color::Rgb(0xa3, 0xbe, 0x8c);

        Self {
            border: nord3,
            title: nord8,
            dim: nord3,
            selection_bg: nord10,
            selection_fg: nord6,
            selection_mod: Modifier::BOLD,
            success: nord14,
            error: nord11,
            warning: nord13,
            modal_bg: nord0,
        }
    }

    fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    fn title_style(&self) -> Style {
        Style::default().fg(self.title).add_modifier(Modifier::BOLD)
    }

    fn dim_style(&self) -> Style {
        Style::default().fg(self.dim)
    }

    fn selected_style(&self) -> Style {
        Style::default()
            .fg(self.selection_fg)
            .bg(self.selection_bg)
            .add_modifier(self.selection_mod)
    }

    fn success_style(&self) -> Style {
        Style::default()
            .fg(self.success)
            .add_modifier(Modifier::BOLD)
    }

    fn error_style(&self) -> Style {
        Style::default().fg(self.error).add_modifier(Modifier::BOLD)
    }

    fn warning_style(&self) -> Style {
        Style::default()
            .fg(self.warning)
            .add_modifier(Modifier::BOLD)
    }

    fn modal_style(&self) -> Style {
        Style::default().bg(self.modal_bg)
    }
}

fn theme() -> &'static Theme {
    static THEME: OnceLock<Theme> = OnceLock::new();
    THEME.get_or_init(|| {
        let term = std::env::var("TERM").unwrap_or_default();
        let no_color = std::env::var_os("NO_COLOR").is_some() || term == "dumb";
        if no_color {
            return Theme::plain();
        }

        match std::env::var("DROIDGEAR_TUI_THEME").ok().as_deref() {
            Some("plain") | Some("none") | Some("no-color") => Theme::plain(),
            Some("nord") | Some("default") | None => Theme::nord(),
            Some(_) => Theme::nord(),
        }
    })
}

fn block<'a>(title: impl Into<ratatui::widgets::block::Title<'a>>) -> Block<'a> {
    let t = theme();
    Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(t.border_style())
        .title_style(t.title_style())
}

pub fn draw(frame: &mut Frame, app: &app::App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(18), Constraint::Min(0)].as_ref())
        .split(area);

    draw_nav(frame, app, chunks[0]);
    draw_main(frame, app, chunks[1]);

    if let Some(modal) = app.modal.as_ref() {
        draw_modal(frame, modal);
    } else if let Some(toast) = app.toast.as_ref() {
        draw_toast(frame, toast);
    }
}

fn draw_nav(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let items: Vec<ListItem> = app::App::nav_items()
        .iter()
        .enumerate()
        .map(|(i, (label, _))| {
            let mut style = Style::default();
            if i == app.nav_index && app.screen == app::Screen::Main {
                style = t.selected_style();
            }
            ListItem::new(Line::from(Span::styled(*label, style)))
        })
        .collect();

    let list = List::new(items).block(block("DroidGear"));
    frame.render_widget(list, area);
}

fn draw_main(frame: &mut Frame, app: &app::App, area: Rect) {
    match app.screen {
        app::Screen::Main => draw_home(frame, area),
        app::Screen::Paths => draw_paths(frame, app, area),
        app::Screen::Factory => draw_factory(frame, app, area),
        app::Screen::FactoryModel => draw_factory_model(frame, app, area),
        app::Screen::Mcp => draw_mcp(frame, app, area),
        app::Screen::McpServer => draw_mcp_server(frame, app, area),
        app::Screen::McpArgs => draw_mcp_args(frame, app, area),
        app::Screen::McpKeyValues => draw_mcp_key_values(frame, app, area),
        app::Screen::Codex => draw_codex_profiles(frame, app, area),
        app::Screen::CodexProfile => draw_codex_profile(frame, app, area),
        app::Screen::CodexProvider => draw_codex_provider(frame, app, area),
        app::Screen::OpenCode => draw_opencode_profiles(frame, app, area),
        app::Screen::OpenCodeProfile => draw_opencode_profile(frame, app, area),
        app::Screen::OpenCodeProvider => draw_opencode_provider(frame, app, area),
        app::Screen::OpenCodeModel => draw_opencode_model(frame, app, area),
        app::Screen::OpenClaw => draw_openclaw_profiles(frame, app, area),
        app::Screen::OpenClawProfile => draw_openclaw_profile(frame, app, area),
        app::Screen::OpenClawProvider => draw_openclaw_provider(frame, app, area),
        app::Screen::OpenClawModel => draw_openclaw_model(frame, app, area),
        app::Screen::OpenClawHelpers => draw_openclaw_helpers(frame, app, area),
        app::Screen::Sessions => draw_sessions(frame, app, area),
        app::Screen::Specs => draw_specs(frame, app, area),
        app::Screen::Channels => draw_channels(frame, app, area),
        app::Screen::ChannelsEdit => draw_channels_edit(frame, app, area),
    }
}

fn draw_home(frame: &mut Frame, area: Rect) {
    let text = vec![
        Line::from("Enter: open module"),
        Line::from("s: module picker"),
        Line::from("Up/Down: navigate"),
        Line::from("q: quit"),
    ];
    let p = Paragraph::new(text)
        .block(block("Home"))
        .wrap(Wrap { trim: true });
    frame.render_widget(p, area);
}

fn draw_paths(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let mut lines: Vec<Line> = Vec::new();
    if let Some(paths) = app.paths.as_ref() {
        let entries = [
            &paths.factory,
            &paths.opencode,
            &paths.opencode_auth,
            &paths.codex,
            &paths.openclaw,
        ];
        for (i, p) in entries.iter().enumerate() {
            let selected = i == app.paths_index;
            let style = if selected {
                t.selected_style()
            } else {
                Style::default()
            };
            let default_tag = if p.is_default { "default" } else { "custom" };
            let tag_style = if selected {
                t.selected_style()
            } else {
                t.dim_style()
            };
            lines.push(Line::from(vec![
                Span::styled(format!("{:>10}: ", p.key), style),
                Span::styled(p.path.clone(), style),
                Span::styled(format!("  [{default_tag}]"), tag_style),
            ]));
        }
    } else {
        lines.push(Line::from("Failed to load paths"));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Enter/e: edit  x: reset  r: refresh  q/Esc: back",
        t.dim_style(),
    )));

    let p = Paragraph::new(lines)
        .block(block("Paths"))
        .wrap(Wrap { trim: false });
    frame.render_widget(p, area);
}

fn draw_factory(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let mut items: Vec<ListItem> = Vec::new();
    for (i, m) in app.custom_models.iter().enumerate() {
        let selected = i == app.factory_models_index;
        let mut style = Style::default();
        if selected {
            style = t.selected_style();
        }
        let name = m
            .display_name
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(&m.model);
        let id =
            m.id.as_deref()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or("-");
        let is_default = app
            .factory_default_model_id
            .as_deref()
            .is_some_and(|d| d == id);
        let default_tag = if is_default { " *" } else { "" };
        items.push(ListItem::new(Line::from(Span::styled(
            format!("{name}  ({id}){default_tag}"),
            style,
        ))));
    }

    if items.is_empty() {
        items.push(ListItem::new(Line::from("No custom models")));
    }

    let title = match app.factory_default_model_id.as_deref() {
        Some(id) => format!("Factory (default model: {id})"),
        None => "Factory (default model: -)".to_string(),
    };

    let list = List::new(items)
        .block(block(title))
        .highlight_style(t.selected_style());
    frame.render_widget(list, area);

    let help = Paragraph::new(vec![Line::from(
        "Up/Down: select  Enter/e: open  n: new  c: copy  x: delete  d: set default  E: raw edit  r: refresh  q/Esc: back",
    )])
    .style(t.dim_style())
    .block(Block::default().borders(Borders::NONE));
    let help_area = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(2),
        width: area.width,
        height: 2,
    };
    frame.render_widget(help, help_area);
}

fn draw_factory_model(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let Some(draft) = app.factory_draft.as_ref() else {
        let p = Paragraph::new(vec![Line::from("No model loaded")])
            .block(block("Factory Model"))
            .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };

    let provider = match draft.provider {
        droidgear_core::factory_settings::Provider::Anthropic => "anthropic",
        droidgear_core::factory_settings::Provider::Openai => "openai",
        droidgear_core::factory_settings::Provider::GenericChatCompletionApi => {
            "generic-chat-completion-api"
        }
    };
    let api_key_set = !draft.api_key.trim().is_empty();
    let max_tokens = draft
        .max_output_tokens
        .map(|v| v.to_string())
        .unwrap_or_else(|| "(default)".to_string());
    let supports_images = draft.supports_images.unwrap_or(false);
    let display_name = draft.display_name.clone().unwrap_or_default();

    let title = if draft.model.trim().is_empty() {
        "Factory Model: (new)".to_string()
    } else {
        format!("Factory Model: {}", draft.model)
    };

    let fields: Vec<(&str, String)> = vec![
        ("Provider", provider.to_string()),
        ("Base URL", draft.base_url.clone()),
        (
            "API Key",
            if api_key_set {
                "********".to_string()
            } else {
                "(not set)".to_string()
            },
        ),
        ("Model", draft.model.clone()),
        (
            "Display Name",
            if display_name.trim().is_empty() {
                "(none)".to_string()
            } else {
                display_name
            },
        ),
        ("Max Tokens", max_tokens),
        (
            "Supports Images",
            if supports_images {
                "yes".to_string()
            } else {
                "no".to_string()
            },
        ),
    ];

    let mut items: Vec<ListItem> = Vec::new();
    for (i, (label, value)) in fields.into_iter().enumerate() {
        let mut style = Style::default();
        if i == app.factory_model_field_index {
            style = t.selected_style();
        }
        items.push(ListItem::new(Line::from(Span::styled(
            format!("{label:>16}: {value}"),
            style,
        ))));
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)].as_ref())
        .split(area);

    let list = List::new(items).block(block(title));
    frame.render_widget(list, chunks[0]);

    let help = Paragraph::new(vec![Line::from(
        "Up/Down: select  Enter/e: edit/toggle  s: save  q/Esc: back",
    )])
    .style(t.dim_style())
    .block(Block::default().borders(Borders::NONE));
    frame.render_widget(help, chunks[1]);
}

fn draw_mcp(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let mut items: Vec<ListItem> = Vec::new();
    for (i, s) in app.mcp_servers.iter().enumerate() {
        let selected = i == app.mcp_index;
        let mut style = Style::default();
        if selected {
            style = t.selected_style();
        }
        let status = if s.config.disabled {
            "disabled"
        } else {
            "enabled"
        };
        items.push(ListItem::new(Line::from(Span::styled(
            format!("{}  [{status}]", s.name),
            style,
        ))));
    }
    if items.is_empty() {
        items.push(ListItem::new(Line::from("No MCP servers")));
    }
    let list = List::new(items)
        .block(block("MCP"))
        .highlight_style(t.selected_style());
    frame.render_widget(list, area);

    let help = Paragraph::new(vec![Line::from(
        "Up/Down: select  Enter/e: open  n: new  t: toggle  d: delete  r: refresh  q/Esc: back",
    )])
    .style(t.dim_style())
    .block(Block::default().borders(Borders::NONE));
    let help_area = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(2),
        width: area.width,
        height: 2,
    };
    frame.render_widget(help, help_area);
}

fn draw_mcp_server(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let Some(server) = app.mcp_edit_draft.as_ref() else {
        let p = Paragraph::new(vec![Line::from("No server loaded")])
            .block(block("MCP Server"))
            .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };

    let disabled = if server.config.disabled { "yes" } else { "no" };
    let server_type = match server.config.server_type {
        droidgear_core::mcp::McpServerType::Stdio => "stdio",
        droidgear_core::mcp::McpServerType::Http => "http",
    };

    let mut fields: Vec<(&str, String)> = vec![
        ("Name", server.name.clone()),
        ("Type", server_type.to_string()),
        ("Disabled", disabled.to_string()),
    ];

    match server.config.server_type {
        droidgear_core::mcp::McpServerType::Stdio => {
            let args_count = server.config.args.as_ref().map(|v| v.len()).unwrap_or(0);
            let env_count = server.config.env.as_ref().map(|m| m.len()).unwrap_or(0);
            fields.push(("Command", server.config.command.clone().unwrap_or_default()));
            fields.push(("Args", format!("{args_count}")));
            fields.push(("Env", format!("{env_count}")));
        }
        droidgear_core::mcp::McpServerType::Http => {
            let headers_count = server.config.headers.as_ref().map(|m| m.len()).unwrap_or(0);
            fields.push(("URL", server.config.url.clone().unwrap_or_default()));
            fields.push(("Headers", format!("{headers_count}")));
        }
    }

    let mut items: Vec<ListItem> = Vec::new();
    for (i, (label, value)) in fields.into_iter().enumerate() {
        let mut style = Style::default();
        if i == app.mcp_edit_field_index {
            style = t.selected_style();
        }
        items.push(ListItem::new(Line::from(Span::styled(
            format!("{label:>16}: {value}"),
            style,
        ))));
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)].as_ref())
        .split(area);

    let title = if server.name.trim().is_empty() {
        "MCP Server: (new)".to_string()
    } else {
        format!("MCP Server: {}", server.name)
    };
    let list = List::new(items).block(block(title));
    frame.render_widget(list, chunks[0]);

    let help = Paragraph::new(vec![Line::from(
        "Up/Down: select  Enter/e: edit/open/toggle  s: save  q/Esc: back",
    )])
    .style(t.dim_style())
    .block(Block::default().borders(Borders::NONE));
    frame.render_widget(help, chunks[1]);
}

fn draw_mcp_args(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let args = app
        .mcp_edit_draft
        .as_ref()
        .and_then(|s| s.config.args.as_ref())
        .cloned()
        .unwrap_or_default();

    let mut items: Vec<ListItem> = Vec::new();
    for (i, a) in args.iter().enumerate() {
        let mut style = Style::default();
        if i == app.mcp_args_index {
            style = t.selected_style();
        }
        items.push(ListItem::new(Line::from(Span::styled(a.clone(), style))));
    }
    if items.is_empty() {
        items.push(ListItem::new(Line::from("No args")));
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)].as_ref())
        .split(area);

    let list = List::new(items).block(block("MCP Args"));
    frame.render_widget(list, chunks[0]);

    let help = Paragraph::new(vec![Line::from(
        "Up/Down: select  n: add  Enter/e: edit  x: delete  q/Esc: back",
    )])
    .style(t.dim_style())
    .block(Block::default().borders(Borders::NONE));
    frame.render_widget(help, chunks[1]);
}

fn draw_mcp_key_values(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let Some(server) = app.mcp_edit_draft.as_ref() else {
        let p = Paragraph::new(vec![Line::from("No server loaded")])
            .block(block("MCP"))
            .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };

    let (title, map) = match app.mcp_kv_mode {
        app::McpKeyValuesMode::Env => ("MCP Env", server.config.env.as_ref()),
        app::McpKeyValuesMode::Headers => ("MCP Headers", server.config.headers.as_ref()),
    };
    let mut entries: Vec<(String, String)> = map
        .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        .unwrap_or_default();
    entries.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

    let mut items: Vec<ListItem> = Vec::new();
    for (i, (k, v)) in entries.iter().enumerate() {
        let mut style = Style::default();
        if i == app.mcp_kv_index {
            style = t.selected_style();
        }
        items.push(ListItem::new(Line::from(Span::styled(
            format!("{k}={v}"),
            style,
        ))));
    }
    if items.is_empty() {
        items.push(ListItem::new(Line::from("No entries")));
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)].as_ref())
        .split(area);

    let list = List::new(items).block(block(title));
    frame.render_widget(list, chunks[0]);

    let help = Paragraph::new(vec![Line::from(
        "Up/Down: select  n: add  Enter/e: edit  x: delete  q/Esc: back",
    )])
    .style(t.dim_style())
    .block(Block::default().borders(Borders::NONE));
    frame.render_widget(help, chunks[1]);
}

fn draw_codex_profiles(frame: &mut Frame, app: &app::App, area: Rect) {
    let active = app.codex_active_id.as_deref();
    let selected_index = app.codex_index;
    draw_profile_list(
        frame,
        area,
        "Codex Profiles",
        app.codex_profiles
            .iter()
            .map(|p| (p.name.as_str(), p.id.as_str())),
        active,
        selected_index,
        "Up/Down: select  Enter/e: open  E: raw edit  p: preview  a: apply  n: new  c: copy  d: delete  r: refresh  q/Esc: back",
    );
}

fn draw_opencode_profiles(frame: &mut Frame, app: &app::App, area: Rect) {
    let active = app.opencode_active_id.as_deref();
    let selected_index = app.opencode_index;
    draw_profile_list(
        frame,
        area,
        "OpenCode Profiles",
        app.opencode_profiles
            .iter()
            .map(|p| (p.name.as_str(), p.id.as_str())),
        active,
        selected_index,
        "Up/Down: select  Enter/e: open  E: raw edit  p: preview  a: apply  n: new  c: copy  d: delete  r: refresh  q/Esc: back",
    );
}

fn draw_openclaw_profiles(frame: &mut Frame, app: &app::App, area: Rect) {
    let active = app.openclaw_active_id.as_deref();
    let selected_index = app.openclaw_index;
    draw_profile_list(
        frame,
        area,
        "OpenClaw Profiles",
        app.openclaw_profiles
            .iter()
            .map(|p| (p.name.as_str(), p.id.as_str())),
        active,
        selected_index,
        "Up/Down: select  Enter/e: open  E: raw edit  p: preview  a: apply  n: new  c: copy  d: delete  r: refresh  q/Esc: back",
    );
}

fn draw_openclaw_profile(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let Some(profile) = app.openclaw_detail.as_ref() else {
        let p = Paragraph::new(vec![Line::from("Failed to load profile")])
            .block(block("OpenClaw Profile"))
            .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(28),
                Constraint::Percentage(26),
                Constraint::Min(0),
                Constraint::Length(2),
            ]
            .as_ref(),
        )
        .split(area);

    let fields: Vec<(&str, String)> = vec![
        ("Name", profile.name.clone()),
        (
            "Description",
            profile
                .description
                .clone()
                .unwrap_or_else(|| "".to_string()),
        ),
        (
            "Default Model",
            profile
                .default_model
                .clone()
                .unwrap_or_else(|| "(none)".to_string()),
        ),
    ];

    let mut field_items: Vec<ListItem> = Vec::new();
    for (i, (label, value)) in fields.into_iter().enumerate() {
        let mut style = Style::default();
        if app.openclaw_detail_focus == app::OpenClawProfileFocus::Fields
            && i == app.openclaw_detail_field_index
        {
            style = t.selected_style();
        }
        field_items.push(ListItem::new(Line::from(Span::styled(
            format!("{label:>14}: {value}"),
            style,
        ))));
    }
    let field_list =
        List::new(field_items).block(block(format!("OpenClaw Profile: {}", profile.name)));
    frame.render_widget(field_list, chunks[0]);

    let failovers = profile.failover_models.as_deref().unwrap_or(&[]);
    let mut failover_items: Vec<ListItem> = Vec::new();
    for (i, r) in failovers.iter().enumerate() {
        let mut style = Style::default();
        if app.openclaw_detail_focus == app::OpenClawProfileFocus::Failover
            && i == app.openclaw_detail_failover_index
        {
            style = t.selected_style();
        }
        failover_items.push(ListItem::new(Line::from(Span::styled(r.clone(), style))));
    }
    if failover_items.is_empty() {
        failover_items.push(ListItem::new(Line::from("No failover models")));
    }
    let failover_list = List::new(failover_items).block(block("Failover Models (Tab to focus)"));
    frame.render_widget(failover_list, chunks[1]);

    let mut provider_items: Vec<ListItem> = Vec::new();
    for (i, pid) in app.openclaw_detail_provider_ids.iter().enumerate() {
        let mut style = Style::default();
        if app.openclaw_detail_focus == app::OpenClawProfileFocus::Providers
            && i == app.openclaw_detail_provider_index
        {
            style = t.selected_style();
        }
        provider_items.push(ListItem::new(Line::from(Span::styled(pid.clone(), style))));
    }
    if provider_items.is_empty() {
        provider_items.push(ListItem::new(Line::from("No providers")));
    }
    let provider_list = List::new(provider_items).block(block("Providers (Tab to focus)"));
    frame.render_widget(provider_list, chunks[2]);

    let help = Paragraph::new(vec![Line::from(
        "Tab: switch  Up/Down: move  Enter/e: edit/open  n: add  d: delete  l: load live  h: helpers  p: preview  a: apply  q/Esc: back",
    )])
    .style(t.dim_style())
    .block(Block::default().borders(Borders::NONE));
    frame.render_widget(help, chunks[3]);
}

fn draw_openclaw_provider(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let Some(profile) = app.openclaw_detail.as_ref() else {
        let p = Paragraph::new(vec![Line::from("Failed to load profile")])
            .block(block("OpenClaw Provider"))
            .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };
    let Some(provider_id) = app.openclaw_provider_id.as_deref() else {
        let p = Paragraph::new(vec![Line::from("No provider selected")])
            .block(block("OpenClaw Provider"))
            .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };
    let Some(config) = profile.providers.get(provider_id) else {
        let p = Paragraph::new(vec![Line::from("Provider not found")])
            .block(block("OpenClaw Provider"))
            .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(45),
                Constraint::Min(0),
                Constraint::Length(2),
            ]
            .as_ref(),
        )
        .split(area);

    let api_key_set = config
        .api_key
        .as_deref()
        .is_some_and(|k| !k.trim().is_empty());
    let api_type = config.api.as_deref().unwrap_or("openai-completions");

    let provider_fields: Vec<(&str, String)> = vec![
        (
            "Base URL",
            config.base_url.clone().unwrap_or_else(|| "".to_string()),
        ),
        (
            "API Key",
            if api_key_set {
                "********".to_string()
            } else {
                "(not set)".to_string()
            },
        ),
        ("API Type", api_type.to_string()),
    ];

    let mut field_items: Vec<ListItem> = Vec::new();
    for (i, (label, value)) in provider_fields.into_iter().enumerate() {
        let mut style = Style::default();
        if app.openclaw_provider_focus == app::CodexDetailFocus::Fields
            && i == app.openclaw_provider_field_index
        {
            style = t.selected_style();
        }
        field_items.push(ListItem::new(Line::from(Span::styled(
            format!("{label:>14}: {value}"),
            style,
        ))));
    }
    let fields_list =
        List::new(field_items).block(block(format!("OpenClaw Provider: {provider_id}")));
    frame.render_widget(fields_list, chunks[0]);

    let mut model_items: Vec<ListItem> = Vec::new();
    for (i, m) in config.models.iter().enumerate() {
        let mut style = Style::default();
        if app.openclaw_provider_focus == app::CodexDetailFocus::Providers
            && i == app.openclaw_provider_model_index
        {
            style = t.selected_style();
        }
        model_items.push(ListItem::new(Line::from(Span::styled(m.id.clone(), style))));
    }
    if model_items.is_empty() {
        model_items.push(ListItem::new(Line::from("No models")));
    }
    let models_list = List::new(model_items).block(block("Models (Tab to focus)"));
    frame.render_widget(models_list, chunks[1]);

    let help = Paragraph::new(vec![Line::from(
        "Tab: switch  Up/Down: move  Enter/e: edit/open  n: add model  d: delete model  q/Esc: back",
    )])
    .style(t.dim_style())
    .block(Block::default().borders(Borders::NONE));
    frame.render_widget(help, chunks[2]);
}

fn draw_openclaw_model(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let Some(profile) = app.openclaw_detail.as_ref() else {
        let p = Paragraph::new(vec![Line::from("Failed to load profile")])
            .block(block("OpenClaw Model"))
            .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };
    let Some(provider_id) = app.openclaw_provider_id.as_deref() else {
        let p = Paragraph::new(vec![Line::from("No provider selected")])
            .block(block("OpenClaw Model"))
            .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };
    let Some(provider) = profile.providers.get(provider_id) else {
        let p = Paragraph::new(vec![Line::from("Provider not found")])
            .block(block("OpenClaw Model"))
            .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };
    let Some(model) = provider.models.get(app.openclaw_provider_model_index) else {
        let p = Paragraph::new(vec![Line::from("Model not found")])
            .block(block("OpenClaw Model"))
            .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };

    let input_text = model.input.iter().any(|t| t == "text");
    let input_image = model.input.iter().any(|t| t == "image");

    let fields: Vec<(&str, String)> = vec![
        ("Model ID", model.id.clone()),
        ("Name", model.name.clone().unwrap_or_else(|| "".to_string())),
        (
            "Context Window",
            model
                .context_window
                .map(|v| v.to_string())
                .unwrap_or_default(),
        ),
        (
            "Max Tokens",
            model.max_tokens.map(|v| v.to_string()).unwrap_or_default(),
        ),
        (
            "Reasoning",
            if model.reasoning { "on" } else { "off" }.to_string(),
        ),
        (
            "Input Text",
            if input_text { "on" } else { "off" }.to_string(),
        ),
        (
            "Input Image",
            if input_image { "on" } else { "off" }.to_string(),
        ),
    ];

    let mut items: Vec<ListItem> = Vec::new();
    for (i, (label, value)) in fields.into_iter().enumerate() {
        let mut style = Style::default();
        if i == app.openclaw_model_field_index {
            style = t.selected_style();
        }
        items.push(ListItem::new(Line::from(Span::styled(
            format!("{label:>14}: {value}"),
            style,
        ))));
    }

    let list = List::new(items).block(block("OpenClaw Model"));
    frame.render_widget(list, area);

    let help_area = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(2),
        width: area.width,
        height: 2,
    };
    let help = Paragraph::new(vec![Line::from(
        "Up/Down: select  Enter/e: edit/toggle  q/Esc: back",
    )])
    .style(t.dim_style())
    .block(Block::default().borders(Borders::NONE));
    frame.render_widget(help, help_area);
}

fn draw_openclaw_helpers(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let Some(profile) = app.openclaw_detail.as_ref() else {
        let p = Paragraph::new(vec![Line::from("Failed to load profile")])
            .block(block("OpenClaw Helpers"))
            .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };

    let cfg = profile.block_streaming_config.as_ref();
    let default_mode = cfg
        .and_then(|c| c.block_streaming_default.as_deref())
        .unwrap_or("on");
    let break_mode = cfg
        .and_then(|c| c.block_streaming_break.as_deref())
        .unwrap_or("text_end");
    let min_chars = cfg
        .and_then(|c| c.block_streaming_chunk.as_ref())
        .and_then(|c| c.min_chars)
        .unwrap_or(200);
    let max_chars = cfg
        .and_then(|c| c.block_streaming_chunk.as_ref())
        .and_then(|c| c.max_chars)
        .unwrap_or(600);
    let idle_ms = cfg
        .and_then(|c| c.block_streaming_coalesce.as_ref())
        .and_then(|c| c.idle_ms)
        .unwrap_or(200);
    let telegram_block = cfg
        .and_then(|c| c.telegram_channel.as_ref())
        .and_then(|t| t.block_streaming)
        .unwrap_or(true);
    let telegram_chunk = cfg
        .and_then(|c| c.telegram_channel.as_ref())
        .and_then(|t| t.chunk_mode.as_deref())
        .unwrap_or("newline");

    let fields: Vec<(&str, String)> = vec![
        ("Default", default_mode.to_string()),
        ("Break", break_mode.to_string()),
        ("Min Chars", min_chars.to_string()),
        ("Max Chars", max_chars.to_string()),
        ("Idle Ms", idle_ms.to_string()),
        (
            "Telegram Block",
            if telegram_block { "on" } else { "off" }.to_string(),
        ),
        ("Telegram Mode", telegram_chunk.to_string()),
    ];

    let mut items: Vec<ListItem> = Vec::new();
    for (i, (label, value)) in fields.into_iter().enumerate() {
        let mut style = Style::default();
        if i == app.openclaw_helpers_field_index {
            style = t.selected_style();
        }
        items.push(ListItem::new(Line::from(Span::styled(
            format!("{label:>14}: {value}"),
            style,
        ))));
    }

    let list = List::new(items).block(block("OpenClaw Helpers"));
    frame.render_widget(list, area);

    let help_area = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(2),
        width: area.width,
        height: 2,
    };
    let help = Paragraph::new(vec![Line::from(
        "Up/Down: select  Enter/e: edit/toggle  x: reset defaults  q/Esc: back",
    )])
    .style(t.dim_style())
    .block(Block::default().borders(Borders::NONE));
    frame.render_widget(help, help_area);
}

fn draw_opencode_profile(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let Some(profile) = app.opencode_detail.as_ref() else {
        let p = Paragraph::new(vec![Line::from("Failed to load profile")])
            .block(block("OpenCode Profile"))
            .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(45),
                Constraint::Min(0),
                Constraint::Length(2),
            ]
            .as_ref(),
        )
        .split(area);

    let fields: Vec<(&str, String)> = vec![
        ("Name", profile.name.clone()),
        (
            "Description",
            profile
                .description
                .clone()
                .unwrap_or_else(|| "".to_string()),
        ),
    ];

    let mut lines: Vec<Line> = Vec::new();
    for (i, (label, value)) in fields.into_iter().enumerate() {
        let mut style = Style::default();
        if app.opencode_detail_focus == app::CodexDetailFocus::Fields
            && i == app.opencode_detail_field_index
        {
            style = t.selected_style();
        }
        lines.push(Line::from(vec![
            Span::styled(format!("{label:>16}: "), style),
            Span::raw(value),
        ]));
    }

    let fields_block = Paragraph::new(lines)
        .block(block(format!("OpenCode Profile: {}", profile.name)))
        .wrap(Wrap { trim: false });
    frame.render_widget(fields_block, chunks[0]);

    let mut provider_items: Vec<ListItem> = Vec::new();
    for (i, pid) in app.opencode_detail_provider_ids.iter().enumerate() {
        let mut style = Style::default();
        if app.opencode_detail_focus == app::CodexDetailFocus::Providers
            && i == app.opencode_detail_provider_index
        {
            style = t.selected_style();
        }
        provider_items.push(ListItem::new(Line::from(Span::styled(pid.clone(), style))));
    }
    if provider_items.is_empty() {
        provider_items.push(ListItem::new(Line::from("No providers")));
    }

    let providers_list = List::new(provider_items).block(block("Providers (Tab to focus)"));
    frame.render_widget(providers_list, chunks[1]);

    let help = Paragraph::new(vec![Line::from(
        "Tab: switch  Up/Down: move  Enter/e: edit/open  n: add provider  d: delete provider  i: import live  p: preview  a: apply  q/Esc: back",
    )])
    .style(t.dim_style())
    .block(Block::default().borders(Borders::NONE));
    frame.render_widget(help, chunks[2]);
}

fn draw_opencode_provider(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let Some(profile) = app.opencode_detail.as_ref() else {
        let p = Paragraph::new(vec![Line::from("Failed to load profile")])
            .block(block("OpenCode Provider"))
            .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };
    let Some(provider_id) = app.opencode_provider_id.as_deref() else {
        let p = Paragraph::new(vec![Line::from("No provider selected")])
            .block(block("OpenCode Provider"))
            .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };
    let Some(config) = profile.providers.get(provider_id) else {
        let p = Paragraph::new(vec![Line::from("Provider not found")])
            .block(block("OpenCode Provider"))
            .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(45),
                Constraint::Min(0),
                Constraint::Length(2),
            ]
            .as_ref(),
        )
        .split(area);

    let base_url = config
        .options
        .as_ref()
        .and_then(|o| o.base_url.as_deref())
        .unwrap_or("");
    let timeout = config
        .options
        .as_ref()
        .and_then(|o| o.timeout)
        .map(|t| t.to_string())
        .unwrap_or_else(|| "".to_string());
    let api_key_set = profile
        .auth
        .get(provider_id)
        .and_then(|v| v.get("key"))
        .and_then(|k| k.as_str())
        .is_some_and(|k| !k.trim().is_empty());

    let provider_fields: Vec<(&str, String)> = vec![
        (
            "Display Name",
            config.name.clone().unwrap_or_else(|| "".to_string()),
        ),
        ("NPM", config.npm.clone().unwrap_or_else(|| "".to_string())),
        ("Base URL", base_url.to_string()),
        (
            "API Key",
            if api_key_set {
                "********".to_string()
            } else {
                "(not set)".to_string()
            },
        ),
        ("Timeout", timeout),
    ];

    let mut field_items: Vec<ListItem> = Vec::new();
    for (i, (label, value)) in provider_fields.into_iter().enumerate() {
        let mut style = Style::default();
        if app.opencode_provider_focus == app::CodexDetailFocus::Fields
            && i == app.opencode_provider_field_index
        {
            style = t.selected_style();
        }
        field_items.push(ListItem::new(Line::from(Span::styled(
            format!("{label:>16}: {value}"),
            style,
        ))));
    }
    let fields_list =
        List::new(field_items).block(block(format!("OpenCode Provider: {provider_id}")));
    frame.render_widget(fields_list, chunks[0]);

    let mut model_items: Vec<ListItem> = Vec::new();
    for (i, mid) in app.opencode_provider_model_ids.iter().enumerate() {
        let mut style = Style::default();
        if app.opencode_provider_focus == app::CodexDetailFocus::Providers
            && i == app.opencode_provider_model_index
        {
            style = t.selected_style();
        }
        model_items.push(ListItem::new(Line::from(Span::styled(mid.clone(), style))));
    }
    if model_items.is_empty() {
        model_items.push(ListItem::new(Line::from("No models")));
    }
    let models_list = List::new(model_items).block(block("Models (Tab to focus)"));
    frame.render_widget(models_list, chunks[1]);

    let help = Paragraph::new(vec![Line::from(
        "Tab: switch  Up/Down: move  Enter/e: edit/open  n: add model  d: delete model  q/Esc: back",
    )])
    .style(t.dim_style())
    .block(Block::default().borders(Borders::NONE));
    frame.render_widget(help, chunks[2]);
}

fn draw_opencode_model(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let Some(profile) = app.opencode_detail.as_ref() else {
        let p = Paragraph::new(vec![Line::from("Failed to load profile")])
            .block(block("OpenCode Model"))
            .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };
    let Some(provider_id) = app.opencode_provider_id.as_deref() else {
        let p = Paragraph::new(vec![Line::from("No provider selected")])
            .block(block("OpenCode Model"))
            .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };
    let Some(model_id) = app.opencode_model_id.as_deref() else {
        let p = Paragraph::new(vec![Line::from("No model selected")])
            .block(block("OpenCode Model"))
            .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };
    let model = profile
        .providers
        .get(provider_id)
        .and_then(|p| p.models.as_ref())
        .and_then(|m| m.get(model_id));
    let Some(model) = model else {
        let p = Paragraph::new(vec![Line::from("Model not found")])
            .block(block("OpenCode Model"))
            .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };

    let context = model
        .limit
        .as_ref()
        .and_then(|l| l.context)
        .map(|v| v.to_string())
        .unwrap_or_else(|| "".to_string());
    let output = model
        .limit
        .as_ref()
        .and_then(|l| l.output)
        .map(|v| v.to_string())
        .unwrap_or_else(|| "".to_string());

    let fields: Vec<(&str, String)> = vec![
        (
            "Display Name",
            model.name.clone().unwrap_or_else(|| "".to_string()),
        ),
        ("Context Limit", context),
        ("Output Limit", output),
    ];

    let mut items: Vec<ListItem> = Vec::new();
    for (i, (label, value)) in fields.into_iter().enumerate() {
        let mut style = Style::default();
        if i == app.opencode_model_field_index {
            style = t.selected_style();
        }
        items.push(ListItem::new(Line::from(Span::styled(
            format!("{label:>16}: {value}"),
            style,
        ))));
    }

    let list = List::new(items).block(block(format!("OpenCode Model: {model_id}")));
    frame.render_widget(list, area);

    let help_area = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(2),
        width: area.width,
        height: 2,
    };
    let help = Paragraph::new(vec![Line::from(
        "Up/Down: select  Enter/e: edit  q/Esc: back",
    )])
    .style(t.dim_style())
    .block(Block::default().borders(Borders::NONE));
    frame.render_widget(help, help_area);
}

fn draw_codex_profile(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let Some(profile) = app.codex_detail.as_ref() else {
        let p = Paragraph::new(vec![Line::from("Failed to load profile")])
            .block(block("Codex Profile"))
            .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(45),
                Constraint::Min(0),
                Constraint::Length(2),
            ]
            .as_ref(),
        )
        .split(area);

    let mut lines: Vec<Line> = Vec::new();
    let effort = profile
        .model_reasoning_effort
        .as_deref()
        .unwrap_or("(none)");
    let api_key_set = profile
        .api_key
        .as_deref()
        .is_some_and(|k| !k.trim().is_empty());

    let fields: Vec<(&str, String)> = vec![
        ("Name", profile.name.clone()),
        (
            "Description",
            profile
                .description
                .clone()
                .unwrap_or_else(|| "".to_string()),
        ),
        ("Model Provider", profile.model_provider.clone()),
        ("Model", profile.model.clone()),
        ("Reasoning Effort", effort.to_string()),
        (
            "API Key",
            if api_key_set {
                "********".to_string()
            } else {
                "(not set)".to_string()
            },
        ),
    ];

    for (i, (label, value)) in fields.into_iter().enumerate() {
        let mut style = Style::default();
        if app.codex_detail_focus == app::CodexDetailFocus::Fields
            && i == app.codex_detail_field_index
        {
            style = t.selected_style();
        }
        lines.push(Line::from(vec![
            Span::styled(format!("{label:>16}: "), style),
            Span::raw(value),
        ]));
    }

    let fields_block = Paragraph::new(lines)
        .block(block(format!("Codex Profile: {}", profile.name)))
        .wrap(Wrap { trim: false });
    frame.render_widget(fields_block, chunks[0]);

    let mut provider_items: Vec<ListItem> = Vec::new();
    for (i, pid) in app.codex_detail_provider_ids.iter().enumerate() {
        let mut style = Style::default();
        if app.codex_detail_focus == app::CodexDetailFocus::Providers
            && i == app.codex_detail_provider_index
        {
            style = t.selected_style();
        }
        let active_tag = if pid == &profile.model_provider {
            " *"
        } else {
            ""
        };
        provider_items.push(ListItem::new(Line::from(Span::styled(
            format!("{pid}{active_tag}"),
            style,
        ))));
    }
    if provider_items.is_empty() {
        provider_items.push(ListItem::new(Line::from("No providers")));
    }
    let providers_list = List::new(provider_items).block(block("Providers (Tab to focus)"));
    frame.render_widget(providers_list, chunks[1]);

    let help = Paragraph::new(vec![Line::from(
        "Tab: switch  Up/Down: move  Enter/e: edit/open  n: add provider  s: set active  d: delete provider  l: load live  p: preview  a: apply  q/Esc: back",
    )])
    .style(t.dim_style())
    .block(Block::default().borders(Borders::NONE));
    frame.render_widget(help, chunks[2]);
}

fn draw_codex_provider(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let Some(profile) = app.codex_detail.as_ref() else {
        let p = Paragraph::new(vec![Line::from("Failed to load profile")])
            .block(block("Codex Provider"))
            .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };
    let Some(provider_id) = app.codex_provider_id.as_deref() else {
        let p = Paragraph::new(vec![Line::from("No provider selected")])
            .block(block("Codex Provider"))
            .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };
    let Some(config) = profile.providers.get(provider_id) else {
        let p = Paragraph::new(vec![Line::from("Provider not found")])
            .block(block("Codex Provider"))
            .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)].as_ref())
        .split(area);

    let wire_api = config.wire_api.as_deref().unwrap_or("responses");
    let effort = config.model_reasoning_effort.as_deref().unwrap_or("(none)");
    let api_key_set = config
        .api_key
        .as_deref()
        .is_some_and(|k| !k.trim().is_empty());

    let fields: Vec<(&str, String)> = vec![
        (
            "Name",
            config.name.clone().unwrap_or_else(|| "".to_string()),
        ),
        (
            "Base URL",
            config.base_url.clone().unwrap_or_else(|| "".to_string()),
        ),
        ("Wire API", wire_api.to_string()),
        (
            "Model",
            config.model.clone().unwrap_or_else(|| "".to_string()),
        ),
        ("Reasoning Effort", effort.to_string()),
        (
            "API Key",
            if api_key_set {
                "********".to_string()
            } else {
                "(not set)".to_string()
            },
        ),
    ];

    let mut items: Vec<ListItem> = Vec::new();
    for (i, (label, value)) in fields.into_iter().enumerate() {
        let mut style = Style::default();
        if i == app.codex_provider_field_index {
            style = t.selected_style();
        }
        items.push(ListItem::new(Line::from(Span::styled(
            format!("{label:>16}: {value}"),
            style,
        ))));
    }

    let list = List::new(items).block(block(format!("Codex Provider: {provider_id}")));
    frame.render_widget(list, chunks[0]);

    let help = Paragraph::new(vec![Line::from(
        "Up/Down: select  Enter/e: edit  q/Esc: back",
    )])
    .style(t.dim_style())
    .block(Block::default().borders(Borders::NONE));
    frame.render_widget(help, chunks[1]);
}

fn draw_sessions(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let mut items: Vec<ListItem> = Vec::new();
    for (i, s) in app.sessions.iter().enumerate() {
        let selected = i == app.sessions_index;
        let mut style = Style::default();
        if selected {
            style = t.selected_style();
        }
        items.push(ListItem::new(Line::from(Span::styled(
            format!("{}  [{}]  {}", s.title, s.project, s.model),
            style,
        ))));
    }
    if items.is_empty() {
        items.push(ListItem::new(Line::from("No sessions")));
    }

    let list = List::new(items)
        .block(block("Sessions"))
        .highlight_style(t.selected_style());
    frame.render_widget(list, area);

    let help = Paragraph::new(vec![Line::from(
        "Up/Down: select  Enter/v: view  d: delete  r: refresh  q/Esc: back",
    )])
    .style(t.dim_style())
    .block(Block::default().borders(Borders::NONE));
    let help_area = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(2),
        width: area.width,
        height: 2,
    };
    frame.render_widget(help, help_area);
}

fn draw_specs(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let mut items: Vec<ListItem> = Vec::new();
    for (i, s) in app.specs.iter().enumerate() {
        let selected = i == app.specs_index;
        let mut style = Style::default();
        if selected {
            style = t.selected_style();
        }
        items.push(ListItem::new(Line::from(Span::styled(
            s.name.clone(),
            style,
        ))));
    }
    if items.is_empty() {
        items.push(ListItem::new(Line::from("No specs")));
    }

    let list = List::new(items)
        .block(block("Specs"))
        .highlight_style(t.selected_style());
    frame.render_widget(list, area);

    let help = Paragraph::new(vec![Line::from(
        "Up/Down: select  Enter/e: edit  d: delete  r: refresh  q/Esc: back",
    )])
    .style(t.dim_style())
    .block(Block::default().borders(Borders::NONE));
    let help_area = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(2),
        width: area.width,
        height: 2,
    };
    frame.render_widget(help, help_area);
}

fn draw_channels(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let mut items: Vec<ListItem> = Vec::new();
    for (i, c) in app.channels.iter().enumerate() {
        let selected = i == app.channels_index;
        let mut style = Style::default();
        if selected {
            style = t.selected_style();
        }
        let enabled = if c.enabled { "on" } else { "off" };
        items.push(ListItem::new(Line::from(Span::styled(
            format!("{}  [{}]  {}", c.name, enabled, c.base_url),
            style,
        ))));
    }
    if items.is_empty() {
        items.push(ListItem::new(Line::from("No channels")));
    }

    let list = List::new(items)
        .block(block("Channels"))
        .highlight_style(t.selected_style());
    frame.render_widget(list, area);

    let help = Paragraph::new(vec![Line::from(
        "Up/Down: select  Enter/e: open  n: new  t: toggle  d: delete  E: raw list  A: raw auth  r: refresh  q/Esc: back",
    )])
    .style(t.dim_style())
    .block(Block::default().borders(Borders::NONE));
    let help_area = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(2),
        width: area.width,
        height: 2,
    };
    frame.render_widget(help, help_area);
}

fn draw_channels_edit(frame: &mut Frame, app: &app::App, area: Rect) {
    let t = theme();
    let Some(channel) = app.channels_edit_draft.as_ref() else {
        let p = Paragraph::new(vec![Line::from("No channel loaded")])
            .block(block("Channel"))
            .wrap(Wrap { trim: true });
        frame.render_widget(p, area);
        return;
    };

    let channel_type = match channel.channel_type {
        droidgear_core::channel::ChannelType::NewApi => "new-api",
        droidgear_core::channel::ChannelType::Sub2Api => "sub-2-api",
        droidgear_core::channel::ChannelType::CliProxyApi => "cli-proxy-api",
        droidgear_core::channel::ChannelType::Ollama => "ollama",
        droidgear_core::channel::ChannelType::General => "general",
    };

    let uses_api_key = matches!(
        channel.channel_type,
        droidgear_core::channel::ChannelType::CliProxyApi
            | droidgear_core::channel::ChannelType::Ollama
            | droidgear_core::channel::ChannelType::General
    );
    let enabled = if channel.enabled { "yes" } else { "no" };
    let api_key_set = !app.channels_edit_api_key.trim().is_empty();
    let password_set = !app.channels_edit_password.trim().is_empty();

    let mut fields: Vec<(&str, String)> = vec![
        ("Name", channel.name.clone()),
        ("Type", channel_type.to_string()),
        ("Base URL", channel.base_url.clone()),
        ("Enabled", enabled.to_string()),
    ];

    if uses_api_key {
        fields.push((
            "API Key",
            if api_key_set {
                "********".to_string()
            } else {
                "(not set)".to_string()
            },
        ));
    } else {
        fields.push(("Username", app.channels_edit_username.clone()));
        fields.push((
            "Password",
            if password_set {
                "********".to_string()
            } else {
                "(not set)".to_string()
            },
        ));
    }

    let mut items: Vec<ListItem> = Vec::new();
    for (i, (label, value)) in fields.into_iter().enumerate() {
        let mut style = Style::default();
        if i == app.channels_edit_field_index {
            style = t.selected_style();
        }
        items.push(ListItem::new(Line::from(Span::styled(
            format!("{label:>16}: {value}"),
            style,
        ))));
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)].as_ref())
        .split(area);

    let title = if channel.name.trim().is_empty() {
        "Channel: (new)".to_string()
    } else {
        format!("Channel: {}", channel.name)
    };
    let list = List::new(items).block(block(title));
    frame.render_widget(list, chunks[0]);

    let help = Paragraph::new(vec![Line::from(
        "Up/Down: select  Enter/e: edit/toggle  s: save  q/Esc: back",
    )])
    .style(t.dim_style())
    .block(Block::default().borders(Borders::NONE));
    frame.render_widget(help, chunks[1]);
}

fn draw_profile_list<'a>(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    profiles: impl Iterator<Item = (&'a str, &'a str)>,
    active_id: Option<&str>,
    selected_index: usize,
    help_text: &str,
) {
    let t = theme();
    let mut items: Vec<ListItem> = Vec::new();
    for (i, (name, id)) in profiles.enumerate() {
        let selected = i == selected_index;
        let mut style = Style::default();
        if selected {
            style = t.selected_style();
        }
        let active_tag = if active_id.is_some_and(|a| a == id) {
            " *"
        } else {
            ""
        };
        items.push(ListItem::new(Line::from(Span::styled(
            format!("{name}{active_tag}"),
            style,
        ))));
    }
    if items.is_empty() {
        items.push(ListItem::new(Line::from("No profiles")));
    }

    let list = List::new(items)
        .block(block(title))
        .highlight_style(t.selected_style());
    frame.render_widget(list, area);

    let help = Paragraph::new(vec![Line::from(help_text)])
        .style(t.dim_style())
        .block(Block::default().borders(Borders::NONE));
    let help_area = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(2),
        width: area.width,
        height: 2,
    };
    frame.render_widget(help, help_area);
}

fn draw_modal(frame: &mut Frame, modal: &app::Modal) {
    let t = theme();
    let area = centered_rect(70, 30, frame.area());
    match modal {
        app::Modal::Confirm { message, .. } => {
            let text = vec![
                Line::from(message.as_str()),
                Line::from(""),
                Line::from(Span::styled("y/Enter: yes    n/Esc: no", t.dim_style())),
            ];
            let block = block("Confirm")
                .border_style(t.warning_style())
                .title_style(t.warning_style());
            let p = Paragraph::new(text)
                .block(block)
                .wrap(Wrap { trim: false })
                .style(t.modal_style());
            frame.render_widget(p, area);
        }
        app::Modal::Input {
            title,
            value,
            is_secret,
            ..
        } => {
            let body = if *is_secret {
                "*".repeat(value.chars().count())
            } else {
                value.clone()
            };
            let text = vec![
                Line::from(body),
                Line::from(""),
                Line::from(Span::styled(
                    "Type, Backspace, Enter to confirm, Esc to cancel",
                    t.dim_style(),
                )),
            ];
            let block = block(title.as_str());
            let p = Paragraph::new(text)
                .block(block)
                .wrap(Wrap { trim: false })
                .style(t.modal_style());
            frame.render_widget(p, area);
        }
        app::Modal::Select {
            title,
            options,
            index,
            ..
        } => {
            let mut lines: Vec<Line> = Vec::new();
            if options.is_empty() {
                lines.push(Line::from("(no options)"));
            } else {
                for (i, opt) in options.iter().enumerate() {
                    let mut style = Style::default();
                    if i == *index {
                        style = t.selected_style();
                    }
                    lines.push(Line::from(Span::styled(opt.clone(), style)));
                }
            }
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Up/Down: select    Enter: confirm    Esc: cancel",
                t.dim_style(),
            )));
            let block = block(title.as_str());
            let p = Paragraph::new(lines)
                .block(block)
                .wrap(Wrap { trim: false })
                .style(t.modal_style());
            frame.render_widget(p, area);
        }
    }
}

fn draw_toast(frame: &mut Frame, toast: &app::Toast) {
    let t = theme();
    let area = Rect {
        x: 0,
        y: frame.area().height.saturating_sub(1),
        width: frame.area().width,
        height: 1,
    };
    let style = if toast.is_error {
        t.error_style()
    } else {
        t.success_style()
    };
    let p = Paragraph::new(Line::from(Span::styled(&toast.message, style)))
        .block(Block::default().borders(Borders::NONE));
    frame.render_widget(p, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
