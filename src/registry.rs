#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddressTrust {
    Trusted,
    Known,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddressKind {
    Token,
    Router,
    Exchange,
    Wallet,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct AddressInfo {
    pub address: String,
    pub label: String,
    pub trust: AddressTrust,
    pub kind: AddressKind,
}

pub fn lookup_address(address: &str) -> AddressInfo {
    let normalized = normalize_address(address);

    match normalized.as_str() {
        // Ethereum mainnet USDC
        "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48" => AddressInfo {
            address: normalized,
            label: "USDC Token".to_string(),
            trust: AddressTrust::Known,
            kind: AddressKind::Token,
        },

        // Ethereum mainnet WETH
        "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2" => AddressInfo {
            address: normalized,
            label: "WETH Token".to_string(),
            trust: AddressTrust::Known,
            kind: AddressKind::Token,
        },

        // Uniswap V2 Router
        "0x7a250d5630b4cf539739df2c5dacb4c659f2488d" => AddressInfo {
            address: normalized,
            label: "Uniswap V2 Router".to_string(),
            trust: AddressTrust::Trusted,
            kind: AddressKind::Router,
        },

        // Uniswap Universal Router
        "0x3fc91a3afd70395cd496c647d5a6cc9d4b2b7fad" => AddressInfo {
            address: normalized,
            label: "Uniswap Universal Router".to_string(),
            trust: AddressTrust::Trusted,
            kind: AddressKind::Router,
        },

        // Example trusted demo wallet
        "0x9999999999999999999999999999999999999999" => AddressInfo {
            address: normalized,
            label: "Trusted demo wallet".to_string(),
            trust: AddressTrust::Trusted,
            kind: AddressKind::Wallet,
        },

        _ => AddressInfo {
            address: normalized,
            label: "Unknown address".to_string(),
            trust: AddressTrust::Unknown,
            kind: AddressKind::Unknown,
        },
    }
}

pub fn is_trusted(address: &str) -> bool {
    lookup_address(address).trust == AddressTrust::Trusted
}

pub fn is_known(address: &str) -> bool {
    matches!(
        lookup_address(address).trust,
        AddressTrust::Trusted | AddressTrust::Known
    )
}

fn normalize_address(address: &str) -> String {
    let lower = address.to_ascii_lowercase();

    if lower.starts_with("0x") {
        lower
    } else {
        format!("0x{}", lower)
    }
}
