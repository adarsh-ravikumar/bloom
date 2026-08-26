use bloom_api::{BloomError, CommandRx, FrontendEvent, FrontendRx, FrontendTx};
use crossbeam_channel::select;
use tauri::{AppHandle, Emitter, State};

pub fn handle_inbound_terminal(
    app_handle: AppHandle,
    command_rx: CommandRx,
    frontend_rx: FrontendRx,
) -> Result<(), BloomError> {
    loop {
        let FrontendEvent::Connected = frontend_rx
            .recv()
            .map_err(|_| BloomError::ChannelRecieveError)?
        else {
            continue;
        };

        loop {
            select! {
                recv(command_rx) -> command => {
                    let Ok(command) = command else {
                        // return Err(BloomError::ChannelRecieveError)
                        continue
                    };

                    let command_name = command.command.as_str();
                    println!("Emitting {command_name}");
                    let _ = app_handle.emit(command_name, command.data);
                }

                recv(frontend_rx) -> msg => {
                    let Ok(FrontendEvent::Disconnected) = msg else {
                        continue
                    };

                    break;
                }
            }
        }
    }
}

#[tauri::command]
pub fn frontend_connected(frontend_tx: State<'_, FrontendTx>) {
    println!("Frontend Connected :)");
    let _ = frontend_tx.send(FrontendEvent::Connected);
}

#[tauri::command]
pub fn frontend_disconnected(frontend_tx: State<'_, FrontendTx>) {
    println!("Frontend Disconnected :(");
    let _ = frontend_tx.send(FrontendEvent::Disconnected);
}
