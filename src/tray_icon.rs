use eframe::Frame;
use tray_icon::{
    menu::{ContextMenu, MenuEvent, MenuId, Menu, MenuItem},
    ClickType, TrayIcon, TrayIconBuilder, TrayIconEvent, Icon,
};

pub struct TrayIconData {
    tray_icon: TrayIcon,
    menu_ids: MenuIds,
}

pub struct MenuIds {
	show_id: MenuId,
    quit_id: MenuId,
}

const TRAY_ICON_BYTES: &'static[u8; 25640] = include_bytes!("../assets/eagle.png");

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
		.with_icon(Icon::from_path("assets/favicon.ico", None).unwrap())
        .build()
        .unwrap();

    frame.set_visible(false);

    TrayIconData {
        tray_icon: icon,
        menu_ids: MenuIds {
			show_id: show_item.id().clone(),
            quit_id: quit_item.id().clone(),
        },
    }
}

pub fn handle_events(frame: &mut Frame, tray_icon_data: TrayIconData) -> Option<TrayIconData> {
    match TrayIconEvent::receiver().try_recv() {
        Ok(TrayIconEvent {
            click_type: ClickType::Double,
            ..
        }) => {
			println!("Making visible");
            frame.set_visible(true);
			return None;
        }
        _ => (),
    }

    match MenuEvent::receiver().try_recv() {
        Ok(MenuEvent { id: menu_item_id }) => {
			if menu_item_id == tray_icon_data.menu_ids.quit_id {
				frame.close();
				return None;
			} else if menu_item_id == tray_icon_data.menu_ids.show_id {
				frame.set_visible(true);
				return None;
			}
		}
        _ => (),
    }

	return Some(tray_icon_data);
}
