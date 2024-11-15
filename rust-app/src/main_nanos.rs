use crate::handle_apdu::handle_apdu;
use crate::implementation::*;
use crate::interface::*;
use crate::menu::*;
use crate::settings::*;

use ledger_device_sdk::io;
use ledger_device_sdk::uxapp::{UxEvent, BOLOS_UX_OK};
use ledger_log::{info, trace};
use ledger_prompts_ui::{handle_menu_button_event, show_menu};

#[allow(dead_code)]
pub fn app_main() {
    let mut comm = io::Comm::new();
    let mut states = ParsersState::NoState;
    let mut idle_menu = IdleMenuWithSettings {
        idle_menu: IdleMenu::AppMain,
        settings: Settings,
    };
    let mut busy_menu = BusyMenu::Working;

    info!("Alamgu Example {}", env!("CARGO_PKG_VERSION"));
    info!(
        "State sizes\ncomm: {}\nstates: {}",
        core::mem::size_of::<io::Comm>(),
        core::mem::size_of::<ParsersState>()
    );

    let menu = |states: &ParsersState, idle: &IdleMenuWithSettings, busy: &BusyMenu| match states {
        ParsersState::NoState => show_menu(idle),
        _ => show_menu(busy),
    };

    // Draw some 'welcome' screen
    menu(&states, &idle_menu, &busy_menu);
    loop {
        // Wait for either a specific button push to exit the app
        // or an APDU command
        match comm.next_event::<Ins>() {
            io::Event::Command(ins) => {
                trace!("Command received");
                match handle_apdu(&mut comm, ins, &mut states) {
                    Ok(()) => {
                        trace!("APDU accepted; sending response");
                        comm.reply_ok();
                        trace!("Replied");
                    }
                    Err(sw) => comm.reply(sw),
                };
                // Reset BusyMenu if we are done handling APDU
                if let ParsersState::NoState = states {
                    busy_menu = BusyMenu::Working;
                }
                menu(&states, &idle_menu, &busy_menu);
                trace!("Command done");
            }
            io::Event::Button(btn) => {
                trace!("Button received");
                match states {
                    ParsersState::NoState => {
                        if let Some(DoExitApp) = handle_menu_button_event(&mut idle_menu, btn) {
                            info!("Exiting app at user direction via root menu");
                            ledger_device_sdk::exit_app(0)
                        }
                    }
                    _ => {
                        if let Some(DoCancel) = handle_menu_button_event(&mut busy_menu, btn) {
                            info!("Resetting at user direction via busy menu");
                            reset_parsers_state(&mut states)
                        }
                    }
                };
                menu(&states, &idle_menu, &busy_menu);
                trace!("Button done");
            }
            io::Event::Ticker => {
                if UxEvent::Event.request() != BOLOS_UX_OK {
                    UxEvent::block();
                    // Redisplay application menu here
                    menu(&states, &idle_menu, &busy_menu);
                }
                //trace!("Ignoring ticker event");
            }
        }
    }
}
