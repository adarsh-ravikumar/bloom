use bloom_api::{
    BloomError, CommandChannel, FrontendChannel, FrontendEvent,
    state::global_state,
};
use crossbeam_channel::select;
use tauri::{AppHandle, Emitter, State};

pub fn handle_inbound_terminal(
    app_handle: AppHandle,
    command_channel: CommandChannel,
    frontend_channel: FrontendChannel,
) -> Result<(), BloomError> {
    loop {
        select! {
            recv(command_channel.rx()) -> command => {
                println!("Command channel got a message");
                let Ok(command) = command else {
                    global_state().terminate_app();
                    continue;
                };

                let command_name = command.command.as_str();
                println!("Emitting {command_name}");
                let _ = app_handle.emit(command_name, command.data);
            }

            recv(frontend_channel.rx()) -> msg => {
                println!("Got some message on frontend_rx");
                match msg {
                    Err(_) => (),

                    Ok(FrontendEvent::Disconnected) =>
                        continue,

                    Ok(FrontendEvent::Connected) => {
                        println!("Frontend Connected :) Trying to send client information");

                        let client = global_state().get_client();
                        println!("Client exists? {:?}", client.is_some());

                        if let Some(client) = client {
                            let client = client.into_command()?;
                            println!("Packing into command: {:?}", client);
                            let _ = app_handle.emit(client.command.as_str(), client.data);
                        }
                    }

                };
            }
        }
    }
}

#[tauri::command]
pub fn frontend_connected(frontend_channel: State<'_, FrontendChannel>) {
    frontend_channel.send(FrontendEvent::Connected);
}

#[tauri::command]
pub fn frontend_disconnected(frontend_channel: State<'_, FrontendChannel>) {
    frontend_channel.send(FrontendEvent::Disconnected);
}
