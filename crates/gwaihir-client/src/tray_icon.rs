use eframe::Frame;
use log::info;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuId, MenuItem},
    ClickType, Icon, TrayIcon, TrayIconBuilder, TrayIconEvent,
};

pub struct TrayIconData {
    _tray_icon: TrayIcon, // RAII, need to keep this for the icon to stay in the tray
    menu_ids: MenuIds,
}

pub struct MenuIds {
    show_id: MenuId,
    quit_id: MenuId,
}

const TRAY_ICON_BYTES: &[u8; 1860] = include_bytes!("../assets/eagle_32.png");

pub fn hide_to_tray(frame: &mut Frame) -> TrayIconData {
    let menu = Menu::new();
    let show_item = MenuItem::new("Show", true, None);
    let quit_item = MenuItem::new("Quit", true, None);
    menu.append(&show_item).unwrap();
    menu.append(&quit_item).unwrap();
    // menu.show_context_menu_for_hwnd(hwnd, position)
    let icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Resume the thingy")
        .with_icon(icon_from_png_bytes(TRAY_ICON_BYTES))
        .build()
        .unwrap();

    frame.set_visible(false);

    TrayIconData {
        _tray_icon: icon,
        menu_ids: MenuIds {
            show_id: show_item.id().clone(),
            quit_id: quit_item.id().clone(),
        },
    }
}

pub fn handle_events(frame: &mut Frame, tray_icon_data: TrayIconData) -> Option<TrayIconData> {
    if let Ok(TrayIconEvent {
        click_type: ClickType::Double,
        ..
    }) = TrayIconEvent::receiver().try_recv()
    {
        info!("Making visible");
        frame.set_visible(true);
        return None;
    }

    if let Ok(MenuEvent { id: menu_item_id }) = MenuEvent::receiver().try_recv() {
        if menu_item_id == tray_icon_data.menu_ids.quit_id {
            frame.close();
            return None;
        } else if menu_item_id == tray_icon_data.menu_ids.show_id {
            frame.set_visible(true);
            return None;
        }
    }

    Some(tray_icon_data)
}

fn icon_from_png_bytes(bytes: &[u8]) -> Icon {
    let decoded_icon = lodepng::decode32(bytes).unwrap();
    Icon::from_rgba(
        decoded_icon
            .buffer
            .iter()
            .flat_map(|rgba| rgba.iter())
            .collect(),
        decoded_icon.width.try_into().unwrap(),
        decoded_icon.height.try_into().unwrap(),
    )
    .unwrap()
}
