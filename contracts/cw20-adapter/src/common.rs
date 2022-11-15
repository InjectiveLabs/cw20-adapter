use cosmwasm_std::Addr;
use regex::{Regex};

// TODO: replace with extension
// TODO: optimise so it's compiled once. Or replace with ordinary split

// static TOKEN_FACTORY_EXPR: Regex = Regex::new("factory/([A-Za-z0-9]{44})/([A-Za-z0-9]{44})").unwrap();

fn parser() -> Regex { Regex::new(r"factory/(\w{42})/(\w{42})").unwrap() }
pub fn is_token_factory_denom(denom: &str) -> bool {

    parser().is_match(denom)
}

pub fn get_cw20_address_from_denom(denom: &str) -> Option<&str> {
    let captures = parser().captures(denom)?;
    let cw20addr = captures.get(2)?;
    Some(cw20addr.as_str())
}

pub fn get_denom(adapter_address: &Addr, cw20addr: &Addr) -> String {
    format!("factory/{}/{}", adapter_address, cw20addr)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn it_returns_true_on_correct_token_factory_denom() {
        assert!(!is_token_factory_denom("factory/inj1pvrwmjuusn9wh34j7y520g8gumuy9xtlt6xtzw/inj1n0qvel0zfmsxu3q8q23xzjvuwfxn0ydlhgyh7h"), "input was not treated as token factory denom")
    }

    #[test]
    fn it_returns_false_for_non_token_factory_denom() {
        assert!(!is_token_factory_denom("factory/inj1pvrwmjuusn9wh34j7y520g8gumuy9xtlt6xtzw/inj1n0qvel0zfmsxu3q8q23xzjvuwfxn0ydlhgyh7"), "input was treated as token factory denom")
    }

    #[test]
    fn it_returns_cw_20_address_for_token_factory_denom() {
        assert_eq!(get_cw20_address_from_denom("factory/inj1pvrwmjuusn9wh34j7y520g8gumuy9xtlt6xtzw/inj1n0qvel0zfmsxu3q8q23xzjvuwfxn0ydlhgyh7h").unwrap(), "inj1n0qvel0zfmsxu3q8q23xzjvuwfxn0ydlhgyh7h", "wrong cw20 address returned")
    }

    #[test]
    fn it_returns_none_cw_20_address_for_non_token_factory_denom() {
        assert!(get_cw20_address_from_denom("factory/inj1pvrwmjuusn9wh34j7y520g8gumuy9xtlt6xtzw/inj1n0qvel0zfmsxu3q8q23xzjvuwfxn0ydlhgyh7").is_none(), "cw20 address returned")
    }

    #[test]
    fn it_returns_denom() {
        assert_eq!(get_denom(&Addr::unchecked("inj1pvrwmjuusn9wh34j7y520g8gumuy9xtlt6xtzw".to_string()), &Addr::unchecked("inj1n0qvel0zfmsxu3q8q23xzjvuwfxn0ydlhgyh7h".to_string())), "factory/inj1pvrwmjuusn9wh34j7y520g8gumuy9xtlt6xtzw/inj1n0qvel0zfmsxu3q8q23xzjvuwfxn0ydlhgyh7h", "wrong denom returned")
    }
}