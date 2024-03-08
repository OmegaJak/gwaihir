use egui::ViewportCommand;
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

pub fn hide_to_tray(ctx: &egui::Context) -> TrayIconData {
    let menu = Menu::new();
    let show_item = MenuItem::new("Show", true, None);
    let quit_item = MenuItem::new("Quit", true, None);
    menu.append(&show_item).unwrap();
    menu.append(&quit_item).unwrap();
    let icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Resume the thingy")
        .with_icon(icon_from_png_bytes(TRAY_ICON_BYTES))
        .build()
        .unwrap();

    ctx.send_viewport_cmd(ViewportCommand::Visible(false));

    TrayIconData {
        _tray_icon: icon,
        menu_ids: MenuIds {
            show_id: show_item.id().clone(),
            quit_id: quit_item.id().clone(),
        },
    }
}

pub fn handle_events(ctx: &egui::Context, tray_icon_data: TrayIconData) -> Option<TrayIconData> {
    if let Ok(TrayIconEvent {
        click_type: ClickType::Double,
        ..
    }) = TrayIconEvent::receiver().try_recv()
    {
        info!("Making visible");
        ctx.send_viewport_cmd(ViewportCommand::Visible(true));
        return None;
    }

    if let Ok(MenuEvent { id: menu_item_id }) = MenuEvent::receiver().try_recv() {
        info!("Menu event");
        if menu_item_id == tray_icon_data.menu_ids.quit_id {
            info!("Closing");
            ctx.send_viewport_cmd(ViewportCommand::Close);
            return None;
        }

        if menu_item_id == tray_icon_data.menu_ids.show_id {
            info!("Showing");
            ctx.send_viewport_cmd(ViewportCommand::Visible(true));
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
