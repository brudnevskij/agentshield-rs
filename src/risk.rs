use crate::{
    registry::{AddressTrust, lookup_address},
    types::{AnalyzeResponse, DecodedAction, Finding, Recommendation, RiskLevel},
};

pub fn analyze_risk(decoded_action: DecodedAction) -> AnalyzeResponse {
    match decoded_action {
        DecodedAction::NativeTransfer { recipient, amount } => {
            analyze_native_transfer(recipient, amount)
        }

        DecodedAction::Erc20Approve {
            token,
            spender,
            amount,
        } => analyze_erc20_approval(token, spender, amount),

        DecodedAction::Erc20Transfer {
            token,
            recipient,
            amount,
        } => analyze_erc20_transfer(token, recipient, amount),

        DecodedAction::UnknownCall { target, calldata } => analyze_unknown_call(target, calldata),
    }
}

fn analyze_native_transfer(recipient: String, amount: String) -> AnalyzeResponse {
    let recipient_info = lookup_address(&recipient);

    let mut findings = vec![Finding {
        code: "NATIVE_TRANSFER".to_string(),
        severity: RiskLevel::Low,
        score_impact: 10,
        title: "Native token transfer".to_string(),
        description: "The transaction transfers the chain native asset.".to_string(),
    }];

    let mut score = 10;

    match recipient_info.trust {
        AddressTrust::Trusted => {
            score -= 10;
            findings.push(Finding {
                code: "TRUSTED_RECIPIENT".to_string(),
                severity: RiskLevel::Low,
                score_impact: -10,
                title: "Trusted recipient".to_string(),
                description: format!("Recipient is known as {}.", recipient_info.label),
            });
        }
        AddressTrust::Known => {
            findings.push(Finding {
                code: "KNOWN_RECIPIENT".to_string(),
                severity: RiskLevel::Low,
                score_impact: 0,
                title: "Known recipient".to_string(),
                description: format!("Recipient is known as {}.", recipient_info.label),
            });
        }
        AddressTrust::Unknown => {
            score += 20;
            findings.push(Finding {
                code: "UNKNOWN_RECIPIENT".to_string(),
                severity: RiskLevel::Medium,
                score_impact: 20,
                title: "Unknown recipient".to_string(),
                description: "The recipient is not in the trusted address registry.".to_string(),
            });
        }
    }

    let risk_score = clamp_score(score);

    AnalyzeResponse {
        risk_score,
        risk_level: risk_level_from_score(risk_score),
        recommendation: recommendation_from_score(risk_score),
        summary: "Native token transfer detected".to_string(),
        decoded_action: DecodedAction::NativeTransfer { recipient, amount },
        findings,
    }
}

fn analyze_erc20_approval(token: String, spender: String, amount: String) -> AnalyzeResponse {
    let spender_info = lookup_address(&spender);
    let token_info = lookup_address(&token);

    let mut score = 20;

    let mut findings = vec![Finding {
        code: "ERC20_APPROVAL".to_string(),
        severity: RiskLevel::High,
        score_impact: 20,
        title: "ERC-20 approval detected".to_string(),
        description: "The transaction grants another address permission to spend tokens."
            .to_string(),
    }];

    if token_info.trust != AddressTrust::Unknown {
        findings.push(Finding {
            code: "KNOWN_TOKEN".to_string(),
            severity: RiskLevel::Low,
            score_impact: 0,
            title: "Known token contract".to_string(),
            description: format!("Token contract is known as {}.", token_info.label),
        });
    }

    if amount == "unlimited" {
        score += 50;
        findings.push(Finding {
            code: "UNLIMITED_APPROVAL".to_string(),
            severity: RiskLevel::Critical,
            score_impact: 50,
            title: "Unlimited token approval".to_string(),
            description: "The spender can move all current and future token balance.".to_string(),
        });
    }

    match spender_info.trust {
        AddressTrust::Trusted => {
            score -= 15;
            findings.push(Finding {
                code: "TRUSTED_SPENDER".to_string(),
                severity: RiskLevel::Low,
                score_impact: -15,
                title: "Trusted spender".to_string(),
                description: format!("Spender is known as {}.", spender_info.label),
            });
        }
        AddressTrust::Known => {
            score -= 5;
            findings.push(Finding {
                code: "KNOWN_SPENDER".to_string(),
                severity: RiskLevel::Low,
                score_impact: -5,
                title: "Known spender".to_string(),
                description: format!("Spender is known as {}.", spender_info.label),
            });
        }
        AddressTrust::Unknown => {
            score += 25;
            findings.push(Finding {
                code: "UNKNOWN_SPENDER".to_string(),
                severity: RiskLevel::High,
                score_impact: 25,
                title: "Unknown spender".to_string(),
                description: "The spender is not in the trusted address registry.".to_string(),
            });
        }
    }

    let risk_score = clamp_score(score);

    AnalyzeResponse {
        risk_score,
        risk_level: risk_level_from_score(risk_score),
        recommendation: recommendation_from_score(risk_score),
        summary: approval_summary(&amount, &spender_info.trust),
        decoded_action: DecodedAction::Erc20Approve {
            token,
            spender,
            amount,
        },
        findings,
    }
}

fn analyze_erc20_transfer(token: String, recipient: String, amount: String) -> AnalyzeResponse {
    let recipient_info = lookup_address(&recipient);
    let token_info = lookup_address(&token);

    let mut score = 20;

    let mut findings = vec![Finding {
        code: "ERC20_TRANSFER".to_string(),
        severity: RiskLevel::Medium,
        score_impact: 20,
        title: "ERC-20 transfer detected".to_string(),
        description: "The transaction transfers ERC-20 tokens to another address.".to_string(),
    }];

    if token_info.trust != AddressTrust::Unknown {
        findings.push(Finding {
            code: "KNOWN_TOKEN".to_string(),
            severity: RiskLevel::Low,
            score_impact: 0,
            title: "Known token contract".to_string(),
            description: format!("Token contract is known as {}.", token_info.label),
        });
    }

    match recipient_info.trust {
        AddressTrust::Trusted => {
            score -= 15;
            findings.push(Finding {
                code: "TRUSTED_RECIPIENT".to_string(),
                severity: RiskLevel::Low,
                score_impact: -15,
                title: "Trusted recipient".to_string(),
                description: format!("Recipient is known as {}.", recipient_info.label),
            });
        }
        AddressTrust::Known => {
            score -= 5;
            findings.push(Finding {
                code: "KNOWN_RECIPIENT".to_string(),
                severity: RiskLevel::Low,
                score_impact: -5,
                title: "Known recipient".to_string(),
                description: format!("Recipient is known as {}.", recipient_info.label),
            });
        }
        AddressTrust::Unknown => {
            score += 20;
            findings.push(Finding {
                code: "UNKNOWN_RECIPIENT".to_string(),
                severity: RiskLevel::Medium,
                score_impact: 20,
                title: "Unknown recipient".to_string(),
                description: "The recipient is not in the trusted address registry.".to_string(),
            });
        }
    }

    let risk_score = clamp_score(score);

    AnalyzeResponse {
        risk_score,
        risk_level: risk_level_from_score(risk_score),
        recommendation: recommendation_from_score(risk_score),
        summary: "ERC-20 token transfer detected".to_string(),
        decoded_action: DecodedAction::Erc20Transfer {
            token,
            recipient,
            amount,
        },
        findings,
    }
}

fn analyze_unknown_call(target: Option<String>, calldata: Option<String>) -> AnalyzeResponse {
    let mut score = 40;

    let mut findings = vec![Finding {
        code: "UNKNOWN_CALLDATA".to_string(),
        severity: RiskLevel::High,
        score_impact: 40,
        title: "Unknown calldata".to_string(),
        description: "AgentShield could not decode this transaction calldata.".to_string(),
    }];

    if let Some(target_address) = target.as_ref() {
        let target_info = lookup_address(target_address);

        match target_info.trust {
            AddressTrust::Trusted => {
                score -= 15;
                findings.push(Finding {
                    code: "TRUSTED_TARGET".to_string(),
                    severity: RiskLevel::Low,
                    score_impact: -15,
                    title: "Trusted target".to_string(),
                    description: format!("Target is known as {}.", target_info.label),
                });
            }
            AddressTrust::Known => {
                score -= 5;
                findings.push(Finding {
                    code: "KNOWN_TARGET".to_string(),
                    severity: RiskLevel::Low,
                    score_impact: -5,
                    title: "Known target".to_string(),
                    description: format!("Target is known as {}.", target_info.label),
                });
            }
            AddressTrust::Unknown => {
                score += 25;
                findings.push(Finding {
                    code: "UNKNOWN_CONTRACT".to_string(),
                    severity: RiskLevel::High,
                    score_impact: 25,
                    title: "Unknown contract".to_string(),
                    description: "The target contract is not in the trusted address registry."
                        .to_string(),
                });
            }
        }
    }

    let risk_score = clamp_score(score);

    AnalyzeResponse {
        risk_score,
        risk_level: risk_level_from_score(risk_score),
        recommendation: recommendation_from_score(risk_score),
        summary: "Unknown contract call detected".to_string(),
        decoded_action: DecodedAction::UnknownCall { target, calldata },
        findings,
    }
}

fn approval_summary(amount: &str, spender_trust: &AddressTrust) -> String {
    match (amount == "unlimited", spender_trust) {
        (true, AddressTrust::Unknown) => "Unlimited ERC-20 approval to unknown spender".to_string(),
        (true, _) => "Unlimited ERC-20 approval detected".to_string(),
        (false, AddressTrust::Unknown) => "ERC-20 approval to unknown spender".to_string(),
        (false, _) => "ERC-20 approval detected".to_string(),
    }
}

fn risk_level_from_score(score: u8) -> RiskLevel {
    match score {
        0..=20 => RiskLevel::Low,
        21..=50 => RiskLevel::Medium,
        51..=80 => RiskLevel::High,
        _ => RiskLevel::Critical,
    }
}

fn recommendation_from_score(score: u8) -> Recommendation {
    match score {
        0..=20 => Recommendation::Allow,
        21..=80 => Recommendation::Review,
        _ => Recommendation::Reject,
    }
}

fn clamp_score(score: i32) -> u8 {
    score.clamp(0, 100) as u8
}
