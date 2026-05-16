use crate::types::{AnalyzeResponse, DecodedAction, Finding, Recommendation, RiskLevel};

pub fn analyze_risk(decoded_action: DecodedAction) -> AnalyzeResponse {
    match decoded_action {
        DecodedAction::NativeTransfer { recipient, amount } => AnalyzeResponse {
            risk_score: 15,
            risk_level: RiskLevel::Low,
            recommendation: Recommendation::Allow,
            summary: "Native token transfer detected".to_string(),
            decoded_action: DecodedAction::NativeTransfer { recipient, amount },
            findings: vec![Finding {
                code: "NATIVE_TRANSFER".to_string(),
                severity: RiskLevel::Low,
                score_impact: 10,
                title: "Native token transfer".to_string(),
                description: "The transaction transfers the chain native asset.".to_string(),
            }],
        },

        DecodedAction::Erc20Approve {
            token,
            spender,
            amount,
        } => analyze_erc20_approval(token, spender, amount),

        DecodedAction::Erc20Transfer {
            token,
            recipient,
            amount,
        } => AnalyzeResponse {
            risk_score: 35,
            risk_level: RiskLevel::Medium,
            recommendation: Recommendation::Review,
            summary: "ERC-20 token transfer detected".to_string(),
            decoded_action: DecodedAction::Erc20Transfer {
                token,
                recipient,
                amount,
            },
            findings: vec![Finding {
                code: "ERC20_TRANSFER".to_string(),
                severity: RiskLevel::Medium,
                score_impact: 20,
                title: "ERC-20 transfer detected".to_string(),
                description: "The transaction transfers ERC-20 tokens to another address."
                    .to_string(),
            }],
        },

        DecodedAction::UnknownCall { target, calldata } => AnalyzeResponse {
            risk_score: 70,
            risk_level: RiskLevel::High,
            recommendation: Recommendation::Review,
            summary: "Unknown contract call detected".to_string(),
            decoded_action: DecodedAction::UnknownCall { target, calldata },
            findings: vec![Finding {
                code: "UNKNOWN_CALLDATA".to_string(),
                severity: RiskLevel::High,
                score_impact: 40,
                title: "Unknown calldata".to_string(),
                description: "AgentShield could not decode this transaction calldata.".to_string(),
            }],
        },
    }
}

fn analyze_erc20_approval(token: String, spender: String, amount: String) -> AnalyzeResponse {
    let is_unlimited = amount == "unlimited";

    let mut findings = vec![Finding {
        code: "ERC20_APPROVAL".to_string(),
        severity: RiskLevel::High,
        score_impact: 20,
        title: "ERC-20 approval detected".to_string(),
        description: "The transaction grants another address permission to spend tokens."
            .to_string(),
    }];

    if is_unlimited {
        findings.push(Finding {
            code: "UNLIMITED_APPROVAL".to_string(),
            severity: RiskLevel::Critical,
            score_impact: 50,
            title: "Unlimited token approval".to_string(),
            description: "The spender can move all current and future token balance.".to_string(),
        });
    }

    let risk_score = if is_unlimited { 90 } else { 55 };

    AnalyzeResponse {
        risk_score,
        risk_level: if is_unlimited {
            RiskLevel::Critical
        } else {
            RiskLevel::High
        },
        recommendation: if is_unlimited {
            Recommendation::Reject
        } else {
            Recommendation::Review
        },
        summary: if is_unlimited {
            "Unlimited ERC-20 approval detected".to_string()
        } else {
            "ERC-20 approval detected".to_string()
        },
        decoded_action: DecodedAction::Erc20Approve {
            token,
            spender,
            amount,
        },
        findings,
    }
}
