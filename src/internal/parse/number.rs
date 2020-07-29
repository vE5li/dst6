use internal::*;
use debug::*;

pub fn parse_number(source: &SharedString, float_source: Option<&SharedString>, negative: bool) -> Status<Option<Data>> {
    let value = match source.printable().parse::<i64>() {
        Ok(value) => Some(value),
        Err(_) => None,
    };

    if let Some(float_source) = float_source {
        match float_source.printable().parse::<i64>() {

            Ok(float_value) => {
                let value = expect!(value, InvalidNumber, identifier!("decimal"));
                let temp = float_value as f64 / (10_u64.pow(float_source.len() as u32) as f64);
                match negative {
                    true => return success!(Some(float!(-(value as f64 + temp)))),
                    false => return success!(Some(float!(value as f64 + temp))),
                }
            },

            Err(_) => return error!(InvalidNumber, identifier!("decimal")),
        };
    }

    match value {

        Some(value) => {
            match negative {
                true => return success!(Some(integer!(-value))),
                false => return success!(Some(integer!(value))),
            }
        },

        None => return success!(None),
    }
}
