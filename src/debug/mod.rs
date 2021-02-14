#[macro_use]
mod macros;
mod position;
mod status;
mod error;

use internal::*;

pub use self::position::Position;
pub use self::status::Status;
pub use self::error::Error;

fn comma_seperated_list(list: &SharedVector<Data>) -> SharedString {
    let mut string = SharedString::new();
    for (index, item) in list.iter().enumerate() {
        if index == 0 {
            string.push_str(&item.serialize());
        } else if index == list.len() - 1 {
            string.push_str(&format_shared!(" or {}", item.serialize()));
        } else {
            string.push_str(&format_shared!(", {}", item.serialize()));
        }
    }
    return string;
}
