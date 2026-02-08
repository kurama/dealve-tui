use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use crate::message::Message;
use crate::model::{Model, Popup};

pub fn handle_event(model: &Model, poll_duration: std::time::Duration) -> Result<Option<Message>> {
    if !event::poll(poll_duration)? {
        return Ok(Some(Message::Tick));
    }
    if let Event::Key(key) = event::read()? {
        if key.kind == KeyEventKind::Press {
            return Ok(handle_key(model, key.code));
        }
    }
    Ok(None)
}

fn handle_key(model: &Model, code: KeyCode) -> Option<Message> {
    match model.ui.popup {
        Popup::Platform => handle_platform_key(code),
        Popup::Options => handle_options_key(code),
        Popup::Keybinds => handle_keybinds_key(code),
        Popup::PriceFilter => handle_price_filter_key(code),
        Popup::None if model.ui.show_menu => handle_menu_key(code),
        Popup::None if model.filter.active => handle_filter_key(code),
        Popup::None => handle_main_key(code),
    }
}

fn handle_platform_key(code: KeyCode) -> Option<Message> {
    match code {
        KeyCode::Esc => Some(Message::ClosePopup),
        KeyCode::Down | KeyCode::Char('j') => Some(Message::PlatformPopupNext),
        KeyCode::Up | KeyCode::Char('k') => Some(Message::PlatformPopupPrev),
        KeyCode::Enter => Some(Message::PlatformPopupSelect),
        _ => None,
    }
}

fn handle_options_key(code: KeyCode) -> Option<Message> {
    match code {
        KeyCode::Esc => Some(Message::ClosePopup),
        KeyCode::Tab | KeyCode::Right => Some(Message::OptionsNextTab),
        KeyCode::BackTab | KeyCode::Left => Some(Message::OptionsPrevTab),
        KeyCode::Down | KeyCode::Char('j') => Some(Message::OptionsNextItem),
        KeyCode::Up | KeyCode::Char('k') => Some(Message::OptionsPrevItem),
        KeyCode::Char('s') => Some(Message::OptionsToggleSortDirection),
        KeyCode::Enter | KeyCode::Char(' ') => Some(Message::OptionsToggleItem),
        _ => None,
    }
}

fn handle_keybinds_key(code: KeyCode) -> Option<Message> {
    match code {
        KeyCode::Esc => Some(Message::ClosePopup),
        _ => None,
    }
}

fn handle_price_filter_key(code: KeyCode) -> Option<Message> {
    match code {
        KeyCode::Esc => Some(Message::ClosePopup),
        KeyCode::Tab => Some(Message::PriceFilterSwitchField),
        KeyCode::Enter => Some(Message::PriceFilterApply),
        KeyCode::Backspace => Some(Message::PriceFilterPop),
        KeyCode::Char('c') => Some(Message::PriceFilterClear),
        KeyCode::Char(c) => Some(Message::PriceFilterPush(c)),
        _ => None,
    }
}

fn handle_menu_key(code: KeyCode) -> Option<Message> {
    match code {
        KeyCode::Esc => Some(Message::ToggleMenu),
        KeyCode::Char('q') => Some(Message::Quit),
        KeyCode::Down | KeyCode::Char('j') => Some(Message::MenuNext),
        KeyCode::Up | KeyCode::Char('k') => Some(Message::MenuPrevious),
        KeyCode::Enter => Some(Message::MenuSelect),
        _ => None,
    }
}

fn handle_filter_key(code: KeyCode) -> Option<Message> {
    match code {
        KeyCode::Esc => Some(Message::CancelFilter),
        KeyCode::Enter => Some(Message::ConfirmFilter),
        KeyCode::Backspace => Some(Message::FilterPop),
        KeyCode::Char(c) => Some(Message::FilterPush(c)),
        _ => None,
    }
}

fn handle_main_key(code: KeyCode) -> Option<Message> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => Some(Message::ToggleMenu),
        KeyCode::Down | KeyCode::Char('j') => Some(Message::SelectNext),
        KeyCode::Up | KeyCode::Char('k') => Some(Message::SelectPrevious),
        KeyCode::Char('p') => Some(Message::OpenPlatformPopup),
        KeyCode::Char('f') => Some(Message::StartFilter),
        KeyCode::Enter => Some(Message::OpenSelectedDeal),
        KeyCode::Char('r') => Some(Message::RequestRefresh),
        KeyCode::Char('s') => Some(Message::ToggleSortDirection),
        KeyCode::Left => Some(Message::PrevSortCriteria),
        KeyCode::Right => Some(Message::NextSortCriteria),
        KeyCode::Char('c') => Some(Message::ClearFilters),
        KeyCode::Char('$') => Some(Message::OpenPriceFilter),
        _ => None,
    }
}
