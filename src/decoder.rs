use crate::types::{AnalyzeRequest, DecodedAction};

const ERC20_APPROVE_SELECTOR: &str = "095ea7b3";
const ERC20_TRANSFER_SELECTOR: &str = "a9059cbb";
const MAX_UINT256_HEX: &str = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";

pub fn decode_transaction(req: &AnalyzeRequest) -> DecodedAction {
    let data = match req.data.as_deref() {
        Some(data) if !is_empty_calldata(data) => normalize_hex(data),
        _ => {
            return decode_empty_calldata(req);
        }
    };

    if data.starts_with(ERC20_APPROVE_SELECTOR) {
        return decode_erc20_approve(req, &data);
    }

    if data.starts_with(ERC20_TRANSFER_SELECTOR) {
        return decode_erc20_transfer(req, &data);
    }

    DecodedAction::UnknownCall {
        target: req.to.clone(),
        calldata: req.data.clone(),
    }
}

fn decode_empty_calldata(req: &AnalyzeRequest) -> DecodedAction {
    DecodedAction::NativeTransfer {
        recipient: req.to.clone().unwrap_or_else(|| "unknown".to_string()),
        amount: req.value.clone(),
    }
}

fn decode_erc20_approve(req: &AnalyzeRequest, data: &str) -> DecodedAction {
    let args = &data[8..];

    let spender_word = match read_abi_word(args, 0) {
        Some(word) => word,
        None => {
            return unknown_call(req);
        }
    };

    let amount_word = match read_abi_word(args, 1) {
        Some(word) => word,
        None => {
            return unknown_call(req);
        }
    };

    let spender = abi_word_to_address(spender_word);
    let amount = format_amount(amount_word);

    DecodedAction::Erc20Approve {
        token: req.to.clone().unwrap_or_else(|| "unknown".to_string()),
        spender,
        amount,
    }
}

fn decode_erc20_transfer(req: &AnalyzeRequest, data: &str) -> DecodedAction {
    let args = &data[8..];

    let recipient_word = match read_abi_word(args, 0) {
        Some(word) => word,
        None => {
            return unknown_call(req);
        }
    };

    let amount_word = match read_abi_word(args, 1) {
        Some(word) => word,
        None => {
            return unknown_call(req);
        }
    };

    let recipient = abi_word_to_address(recipient_word);
    let amount = format_amount(amount_word);

    DecodedAction::Erc20Transfer {
        token: req.to.clone().unwrap_or_else(|| "unknown".to_string()),
        recipient,
        amount,
    }
}

fn read_abi_word(args: &str, index: usize) -> Option<&str> {
    let start = index * 64;
    let end = start + 64;

    if args.len() < end {
        return None;
    }

    Some(&args[start..end])
}

fn abi_word_to_address(word: &str) -> String {
    let address_hex = &word[24..64];
    format!("0x{}", address_hex)
}

fn format_amount(word: &str) -> String {
    if word.eq_ignore_ascii_case(MAX_UINT256_HEX) {
        return "unlimited".to_string();
    }

    format!("0x{}", word)
}

fn normalize_hex(input: &str) -> String {
    input
        .strip_prefix("0x")
        .unwrap_or(input)
        .to_ascii_lowercase()
}

fn is_empty_calldata(data: &str) -> bool {
    let normalized = normalize_hex(data);
    normalized.is_empty()
}

fn unknown_call(req: &AnalyzeRequest) -> DecodedAction {
    DecodedAction::UnknownCall {
        target: req.to.clone(),
        calldata: req.data.clone(),
    }
}
