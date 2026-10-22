use std::future::Future;

use serde::Serialize;
use tauri::{
    Runtime,
    ipc::{Channel, CommandArg, CommandItem, InvokeError},
};

/// A command's result that arrives after the command returns.
///
/// A command taking a `Reply<T>` runs in call order like any blocking command, and its
/// generated binding resolves with the `T` sent here, or rejects with the error, exactly as an
/// `async` command returning `Result<T, E>` would.
pub struct Reply<T>(Channel<Replied<T>>);

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum Replied<T> {
    Ok(T),
    Err(serde_json::Value),
}

impl<T, E: Into<InvokeError>> From<Result<T, E>> for Replied<T> {
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(value) => Self::Ok(value),
            Err(err) => Self::Err(err.into().0),
        }
    }
}

impl<T: Serialize + Send + 'static> Reply<T> {
    /// Reply with what `result` resolves to, once it does.
    pub fn when<E: Into<InvokeError>>(
        self,
        result: impl Future<Output = Result<T, E>> + Send + 'static,
    ) {
        tauri::async_runtime::spawn(async move {
            let replied = result.await.into();
            let _ = self.0.send(replied);
        });
    }
}

impl<'de, R: Runtime, T> CommandArg<'de, R> for Reply<T> {
    fn from_command(command: CommandItem<'de, R>) -> Result<Self, InvokeError> {
        Channel::from_command(command).map(Self)
    }
}

#[allow(dead_code, non_camel_case_types)]
#[derive(specta::Type)]
#[specta(remote = Reply)]
struct TAURI_SPECTA_REPLY<T>(std::marker::PhantomData<T>);
